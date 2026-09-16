import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { createMemo, emitQuickNoteSaved } from '../lib/api';
import QuickNoteView from '../components/QuickNoteView.vue';
import type { Memo } from '../types/memo';

// 组件在 window 上注册 keydown 监听，测后必须卸载避免跨用例残留
enableAutoUnmount(afterEach);

vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: vi.fn() }));
vi.mock('../lib/api', () => ({ createMemo: vi.fn(), emitQuickNoteSaved: vi.fn() }));

const mockClose = vi.fn().mockResolvedValue(undefined);
const mockGetCurrentWindow = vi.mocked(getCurrentWindow);
const mockCreateMemo = vi.mocked(createMemo);
const mockEmit = vi.mocked(emitQuickNoteSaved);

const savedMemo: Memo = {
  id: 1,
  title: '速记标题',
  content: '速记正文',
  created_at: '2026-09-14T03:15:00.000Z',
  updated_at: '2026-09-14T03:15:00.000Z',
  remind_at: '',
};

beforeEach(() => {
  mockClose.mockReset().mockResolvedValue(undefined);
  mockGetCurrentWindow.mockReset().mockReturnValue({ close: mockClose } as never);
  mockCreateMemo.mockReset();
  mockEmit.mockReset();
});

describe('QuickNoteView', () => {
  it('保存触发 create_memo + emit + 关窗', async () => {
    mockCreateMemo.mockResolvedValue(savedMemo);
    mockEmit.mockResolvedValue(undefined);

    const wrapper = mount(QuickNoteView);
    await wrapper.get('.quicknote__title-input').setValue('速记标题');
    await wrapper.get('.quicknote__content').setValue('速记正文');
    await wrapper.get('.quicknote__save').trigger('click');
    await flushPromises();

    expect(mockCreateMemo).toHaveBeenCalledWith({ title: '速记标题', content: '速记正文' });
    expect(mockEmit).toHaveBeenCalled();
    expect(mockClose).toHaveBeenCalled();
  });

  it('空标题保存被拦截，不调用 create_memo', async () => {
    const wrapper = mount(QuickNoteView);
    await wrapper.get('.quicknote__save').trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('标题不能为空');
    expect(mockCreateMemo).not.toHaveBeenCalled();
  });

  it('Esc 无内容直接关窗，不弹二次确认', async () => {
    const wrapper = mount(QuickNoteView);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await flushPromises();

    expect(mockClose).toHaveBeenCalled();
    expect(wrapper.find('.dialog').exists()).toBe(false);
  });

  it('Esc 有内容弹二次确认，确认丢弃后关窗', async () => {
    const wrapper = mount(QuickNoteView);
    await wrapper.get('.quicknote__title-input').setValue('未保存内容');
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await flushPromises();

    expect(wrapper.get('.dialog').text()).toContain('丢弃未保存的速记？');
    expect(mockClose).not.toHaveBeenCalled();

    const discardBtn = wrapper.get('.dialog').findAll('button').find((b) => b.text().includes('丢弃'));
    await discardBtn!.trigger('click');
    await flushPromises();

    expect(mockClose).toHaveBeenCalled();
    expect(mockCreateMemo).not.toHaveBeenCalled();
  });

  it('二次确认取消留在窗内', async () => {
    const wrapper = mount(QuickNoteView);
    await wrapper.get('.quicknote__title-input').setValue('未保存内容');
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await flushPromises();

    const cancelBtn = wrapper.get('.dialog').findAll('button').find((b) => b.text().includes('取消'));
    await cancelBtn!.trigger('click');
    await flushPromises();

    expect(mockClose).not.toHaveBeenCalled();
    expect(wrapper.find('.dialog').exists()).toBe(false);
  });
});