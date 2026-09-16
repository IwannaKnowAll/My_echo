import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import App from '../App.vue';
import type { Memo } from '../types/memo';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));

const mockInvoke = vi.mocked(invoke);

function makeMemo(id: number, title: string, updatedAt: string): Memo {
  return { id, title, content: '', created_at: updatedAt, updated_at: updatedAt, remind_at: '' };
}

beforeEach(() => {
  mockInvoke.mockReset();
});

describe('App 关键流程', () => {
  it('挂载后调用 list_memos 并渲染列表', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_memos') {
        return Promise.resolve([makeMemo(1, '第一条', '2026-09-14T03:00:00.000Z')]);
      }
      return Promise.resolve([]);
    });
    const wrapper = mount(App);
    await flushPromises();

    expect(mockInvoke).toHaveBeenCalledWith('list_memos');
    expect(wrapper.text()).toContain('第一条');
  });

  it('空标题保存被拦截并提示，不调用 create_memo', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_memos') {
        return Promise.resolve([]);
      }
      return Promise.resolve([]);
    });
    const wrapper = mount(App);
    await flushPromises();

    const newBtn = wrapper.findAll('button').find((b) => b.text().includes('新建备忘录'));
    expect(newBtn).toBeDefined();
    await newBtn!.trigger('click');

    await wrapper.get('form').trigger('submit');
    await flushPromises();

    expect(wrapper.text()).toContain('标题不能为空');
    expect(mockInvoke).not.toHaveBeenCalledWith('create_memo', expect.anything());
  });

  it('新建保存成功后回列表并刷新展示新条目', async () => {
    let list: Memo[] = [];
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_memos') {
        return Promise.resolve(list);
      }
      if (cmd === 'create_memo') {
        const created = makeMemo(1, '新标题', '2026-09-14T05:00:00.000Z');
        list = [created];
        return Promise.resolve(created);
      }
      return Promise.resolve([]);
    });
    const wrapper = mount(App);
    await flushPromises();

    const newBtn = wrapper.findAll('button').find((b) => b.text().includes('新建备忘录'));
    await newBtn!.trigger('click');
    await wrapper.get('input').setValue('新标题');
    await wrapper.get('form').trigger('submit');
    await flushPromises();

    expect(mockInvoke).toHaveBeenCalledWith('create_memo', { input: { title: '新标题', content: '' } });
    expect(wrapper.text()).toContain('新标题');
  });

  it('搜索输入防抖 300ms 后调用 search_memos，清空回切 list_memos', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_memos') {
        return Promise.resolve([]);
      }
      if (cmd === 'search_memos') {
        return Promise.resolve([makeMemo(2, '搜索结果', '2026-09-14T04:00:00.000Z')]);
      }
      return Promise.resolve([]);
    });
    const wrapper = mount(App);
    await flushPromises();
    mockInvoke.mockClear();

    vi.useFakeTimers();
    try {
      const searchInput = wrapper.get('.search__input');
      await searchInput.setValue('关键');

      // 未到 300ms 不触发
      await vi.advanceTimersByTimeAsync(200);
      expect(mockInvoke).not.toHaveBeenCalledWith('search_memos', expect.anything());

      // 满 300ms 触发
      await vi.advanceTimersByTimeAsync(100);
      expect(mockInvoke).toHaveBeenCalledWith('search_memos', { input: { keyword: '关键' } });

      // 清空输入立即回切全量
      await searchInput.setValue('');
      await vi.advanceTimersByTimeAsync(0);
      expect(mockInvoke).toHaveBeenLastCalledWith('list_memos');
    } finally {
      vi.useRealTimers();
    }
  });

  it('编辑不存在的备忘录（E_NOT_FOUND）先展示提示再回列表', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_memos') {
        return Promise.resolve([makeMemo(1, '已删除的条目', '2026-09-14T03:00:00.000Z')]);
      }
      if (cmd === 'get_memo') {
        return Promise.reject({ code: 'E_NOT_FOUND', message: '备忘录不存在' });
      }
      return Promise.resolve([]);
    });
    const wrapper = mount(App);
    await flushPromises();

    await wrapper.get('.item').trigger('click');
    await flushPromises();

    // 先展示提示，再回退到列表视图：提示文案存在、列表区存在、编辑表单消失。
    expect(wrapper.text()).toContain('该备忘录不存在或已被删除');
    expect(wrapper.find('.editor-form').exists()).toBe(false);
    expect(wrapper.find('.memo-list').exists()).toBe(true);
  });

  it('编辑保存时 E_NOT_FOUND 先展示提示再回列表', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_memos') {
        return Promise.resolve([makeMemo(1, '待更新', '2026-09-14T03:00:00.000Z')]);
      }
      if (cmd === 'get_memo') {
        return Promise.resolve(makeMemo(1, '待更新', '2026-09-14T03:00:00.000Z'));
      }
      if (cmd === 'update_memo') {
        return Promise.reject({ code: 'E_NOT_FOUND', message: '备忘录不存在' });
      }
      return Promise.resolve([]);
    });
    const wrapper = mount(App);
    await flushPromises();

    await wrapper.get('.item').trigger('click');
    await flushPromises();

    await wrapper.get('input').setValue('改后的标题');
    await wrapper.get('form').trigger('submit');
    await flushPromises();

    expect(mockInvoke).toHaveBeenCalledWith('update_memo', {
      input: { id: 1, title: '改后的标题', content: '' },
    });
    expect(wrapper.text()).toContain('该备忘录不存在或已被删除');
    expect(wrapper.find('.editor-form').exists()).toBe(false);
    expect(wrapper.find('.memo-list').exists()).toBe(true);
  });

  it('确认删除时 E_NOT_FOUND 先展示提示再回列表', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_memos') {
        return Promise.resolve([makeMemo(1, '待删除', '2026-09-14T03:00:00.000Z')]);
      }
      if (cmd === 'get_memo') {
        return Promise.resolve(makeMemo(1, '待删除', '2026-09-14T03:00:00.000Z'));
      }
      if (cmd === 'delete_memo') {
        return Promise.reject({ code: 'E_NOT_FOUND', message: '备忘录不存在' });
      }
      return Promise.resolve([]);
    });
    const wrapper = mount(App);
    await flushPromises();

    await wrapper.get('.item').trigger('click');
    await flushPromises();

    // 编辑态标题栏「删除」入口
    const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('删除'));
    expect(deleteBtn).toBeDefined();
    await deleteBtn!.trigger('click');
    await flushPromises();

    // 确认弹窗内的「删除」确认按钮
    const confirmBtn = wrapper
      .get('.dialog')
      .findAll('button')
      .find((b) => b.text().includes('删除'));
    expect(confirmBtn).toBeDefined();
    await confirmBtn!.trigger('click');
    await flushPromises();

    expect(mockInvoke).toHaveBeenCalledWith('delete_memo', { input: { id: 1 } });
    expect(wrapper.text()).toContain('该备忘录不存在或已被删除');
    expect(wrapper.find('.editor-form').exists()).toBe(false);
    expect(wrapper.find('.memo-list').exists()).toBe(true);
  });
});