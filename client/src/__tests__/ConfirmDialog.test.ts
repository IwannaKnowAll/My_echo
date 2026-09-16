import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ConfirmDialog from '../components/ConfirmDialog.vue';

describe('ConfirmDialog', () => {
  it('visible=false 时不渲染', () => {
    const wrapper = mount(ConfirmDialog, { props: { visible: false, message: 'm' } });
    expect(wrapper.find('.dialog').exists()).toBe(false);
  });

  it('展示标题与正文', () => {
    const wrapper = mount(ConfirmDialog, {
      props: { visible: true, message: '确定要删除“会议纪要”吗？此操作不可撤销。' },
    });
    expect(wrapper.get('.dialog__title').text()).toBe('删除备忘录');
    expect(wrapper.get('.dialog__message').text()).toBe(
      '确定要删除“会议纪要”吗？此操作不可撤销。',
    );
  });

  it('点击确认触发 confirm 事件', async () => {
    const wrapper = mount(ConfirmDialog, { props: { visible: true, message: 'm' } });
    const confirm = wrapper.findAll('button').find((b) => b.text().includes('删除'));
    expect(confirm).toBeDefined();
    await confirm!.trigger('click');
    expect(wrapper.emitted('confirm')).toHaveLength(1);
  });

  it('点击取消触发 cancel 事件', async () => {
    const wrapper = mount(ConfirmDialog, { props: { visible: true, message: 'm' } });
    const cancel = wrapper.findAll('button').find((b) => b.text().includes('取消'));
    expect(cancel).toBeDefined();
    await cancel!.trigger('click');
    expect(wrapper.emitted('cancel')).toHaveLength(1);
  });

  it('loading 时确认被禁用，点击不触发 confirm', async () => {
    const wrapper = mount(ConfirmDialog, {
      props: { visible: true, message: 'm', loading: true },
    });
    const confirm = wrapper.findAll('button').find((b) => b.text().includes('删除'));
    await confirm!.trigger('click');
    expect(wrapper.emitted('confirm')).toBeUndefined();
  });
});