import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import TagsBar from '../components/TagsBar.vue';
import type { TagWithCount } from '../types/memo';

const tags: TagWithCount[] = [
  { id: 1, name: '工作', created_at: '2026-09-14T03:15:00.000Z', memo_count: 3 },
  { id: 2, name: '生活', created_at: '2026-09-14T03:15:00.000Z', memo_count: 1 },
];

describe('TagsBar', () => {
  it('渲染「全部」与标签列表', () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: null } });
    expect(wrapper.text()).toContain('全部');
    expect(wrapper.text()).toContain('工作');
    expect(wrapper.text()).toContain('生活');
  });

  it('空标签显示空态文案', () => {
    const wrapper = mount(TagsBar, { props: { tags: [], activeTagId: null } });
    expect(wrapper.text()).toContain('全部');
    expect(wrapper.text()).toContain('暂无标签');
  });

  it('选中标签时对应 chip 高亮', () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: 1 } });
    const activeChip = wrapper.findAllComponents({ name: 'TagChip' }).find(
      (c) => c.props('tag').id === 1,
    );
    expect(activeChip?.props('active')).toBe(true);
  });

  it('点击「全部」触发 select null', async () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: 1 } });
    await wrapper.get('.tags-bar__all').trigger('click');
    expect(wrapper.emitted('select')).toEqual([[null]]);
  });

  it('点击标签触发 select tagId', async () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: null } });
    const chips = wrapper.findAllComponents({ name: 'TagChip' });
    await chips[0].trigger('click');
    expect(wrapper.emitted('select')).toEqual([[1]]);
  });

  it('点击 + 进入创建，回车触发 create，空名不触发', async () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: null } });
    await wrapper.get('.tags-bar__add').trigger('click');
    const input = wrapper.get('.tags-bar__input');
    await input.setValue('   ');
    await input.trigger('keydown.enter');
    expect(wrapper.emitted('create')).toBeUndefined();

    await input.setValue('新标签');
    await input.trigger('keydown.enter');
    expect(wrapper.emitted('create')).toEqual([['新标签']]);
  });

  it('标签名超过 30 个码点被拦截并提示', async () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: null } });
    await wrapper.get('.tags-bar__add').trigger('click');
    const input = wrapper.get('.tags-bar__input');
    await input.setValue('a'.repeat(31));
    await input.trigger('keydown.enter');
    expect(wrapper.emitted('create')).toBeUndefined();
    expect(wrapper.text()).toContain('标签名不能超过 30 个字符');
  });

  it('30 个 emoji 计为 30 码点，不被 UTF-16 长度误拦', async () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: null } });
    await wrapper.get('.tags-bar__add').trigger('click');
    const input = wrapper.get('.tags-bar__input');
    const name = '😀'.repeat(30);
    await input.setValue(name);
    await input.trigger('keydown.enter');
    expect(wrapper.emitted('create')).toEqual([[name]]);
  });

  it('点击标签移除按钮转发 remove 事件', async () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: null } });
    const chips = wrapper.findAllComponents({ name: 'TagChip' });
    await chips[0].get('.tag__remove').trigger('click');
    expect(wrapper.emitted('remove')).toEqual([[tags[0]]]);
  });

  it('加载中显示骨架', () => {
    const wrapper = mount(TagsBar, { props: { tags, activeTagId: null, loading: true } });
    expect(wrapper.find('.tags-bar__skeleton').exists()).toBe(true);
  });
});