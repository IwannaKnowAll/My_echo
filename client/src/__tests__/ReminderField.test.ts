import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ReminderField from '../components/ReminderField.vue';

describe('ReminderField', () => {
  it('无提醒时显示「添加提醒」入口', () => {
    const wrapper = mount(ReminderField, {
      props: { memoId: 1, remindAt: '' },
    });
    expect(wrapper.text()).toContain('添加提醒');
  });

  it('已设提醒时显示时间与修改/清除入口', () => {
    const wrapper = mount(ReminderField, {
      props: { memoId: 1, remindAt: new Date(Date.now() + 3600_000).toISOString() },
    });
    expect(wrapper.text()).toContain('修改');
    expect(wrapper.text()).toContain('清除');
  });

  it('选择过去时间被拦截，不触发 change', async () => {
    const wrapper = mount(ReminderField, { props: { memoId: 1, remindAt: '' } });
    await wrapper.get('.reminder__add').trigger('click');

    const input = wrapper.get('.reminder__picker');
    await input.setValue('2000-01-01T00:00');
    await wrapper.findAll('.reminder__link').find((b) => b.text() === '设置')!.trigger('click');

    expect(wrapper.text()).toContain('提醒时间不能早于当前时间');
    expect(wrapper.emitted('change')).toBeUndefined();
  });

  it('选择未到时间触发 change 携带 UTC ISO', async () => {
    const wrapper = mount(ReminderField, { props: { memoId: 1, remindAt: '' } });
    await wrapper.get('.reminder__add').trigger('click');

    const futureLocal = new Date(Date.now() + 7200_000);
    const pad = (n: number) => String(n).padStart(2, '0');
    const localValue = `${futureLocal.getFullYear()}-${pad(futureLocal.getMonth() + 1)}-${pad(
      futureLocal.getDate(),
    )}T${pad(futureLocal.getHours())}:${pad(futureLocal.getMinutes())}`;

    await wrapper.get('.reminder__picker').setValue(localValue);
    await wrapper.findAll('.reminder__link').find((b) => b.text() === '设置')!.trigger('click');

    expect(wrapper.emitted('change')).toHaveLength(1);
    expect(wrapper.emitted('change')![0][0]).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/);
  });

  it('清除触发 change 空串', async () => {
    const wrapper = mount(ReminderField, {
      props: { memoId: 1, remindAt: new Date(Date.now() + 3600_000).toISOString() },
    });
    await wrapper.findAll('.reminder__link').find((b) => b.text() === '清除')!.trigger('click');
    expect(wrapper.emitted('change')).toEqual([['']]);
  });
});