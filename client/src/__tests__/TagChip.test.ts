import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import TagChip from '../components/TagChip.vue';
import type { Tag } from '../types/memo';

const tag: Tag = { id: 1, name: '工作', created_at: '2026-09-14T03:15:00.000Z' };

describe('TagChip', () => {
  it('渲染标签名', () => {
    const wrapper = mount(TagChip, { props: { tag } });
    expect(wrapper.text()).toContain('工作');
  });

  it('active 时追加选中态样式类', () => {
    const wrapper = mount(TagChip, { props: { tag, active: true } });
    expect(wrapper.classes()).toContain('tag--active');
  });

  it('removable 时展示移除按钮并触发 remove（不冒泡 click）', async () => {
    const wrapper = mount(TagChip, { props: { tag, removable: true } });
    const removeBtn = wrapper.get('.tag__remove');
    await removeBtn.trigger('click');
    expect(wrapper.emitted('remove')).toHaveLength(1);
    expect(wrapper.emitted('click')).toBeUndefined();
  });

  it('非 removable 不渲染移除按钮', () => {
    const wrapper = mount(TagChip, { props: { tag } });
    expect(wrapper.find('.tag__remove').exists()).toBe(false);
  });

  it('点击触发 click 事件', async () => {
    const wrapper = mount(TagChip, { props: { tag } });
    await wrapper.trigger('click');
    expect(wrapper.emitted('click')).toHaveLength(1);
  });

  it('disabled 时移除按钮禁用', () => {
    const wrapper = mount(TagChip, { props: { tag, removable: true, disabled: true } });
    expect(wrapper.get('.tag__remove').attributes('disabled')).toBeDefined();
  });
});