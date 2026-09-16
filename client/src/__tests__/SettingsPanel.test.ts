import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils';
import { afterEach, describe, expect, it } from 'vitest';
import SettingsPanel from '../components/SettingsPanel.vue';
import { DEFAULT_SHORTCUTS } from '../lib/shortcuts';
import type { AppSettings } from '../types/memo';

// 组件在录制态会向 window 注册 keydown 监听，测后必须卸载避免跨用例残留
enableAutoUnmount(afterEach);

const settings: AppSettings = {
  theme: 'system',
  close_behavior: 'quit',
  shortcuts: { ...DEFAULT_SHORTCUTS },
};

describe('SettingsPanel', () => {
  it('visible=false 不渲染', () => {
    const wrapper = mount(SettingsPanel, { props: { visible: false, settings } });
    expect(wrapper.find('.settings').exists()).toBe(false);
  });

  it('渲染三块：主题/关闭行为/快捷键', () => {
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings } });
    expect(wrapper.text()).toContain('主题');
    expect(wrapper.text()).toContain('关闭主窗口后保留在菜单栏');
    expect(wrapper.text()).toContain('快捷键');
  });

  it('settings 为 null 时显示加载态', () => {
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings: null } });
    expect(wrapper.text()).toContain('加载中…');
  });

  it('切换主题触发 update theme', async () => {
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings } });
    const seg = wrapper.findAll('.settings__segment').find((b) => b.text() === '深色');
    await seg!.trigger('click');
    expect(wrapper.emitted('update')).toEqual([[{ theme: 'dark' }]]);
  });

  it('切换开关触发 update close_behavior', async () => {
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings } });
    await wrapper.get('.switch').trigger('click');
    expect(wrapper.emitted('update')).toEqual([[{ close_behavior: 'hide' }]]);
  });

  it('录制态按下非法组合提示必须含修饰键', async () => {
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings } });
    const editBtn = wrapper.findAll('.settings__link').find((b) => b.text() === '修改');
    await editBtn!.trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('按下新组合');

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }));
    await flushPromises();
    expect(wrapper.text()).toContain('快捷键必须包含修饰键或为功能键');
  });

  it('录制态按 Esc 取消录制', async () => {
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings } });
    const editBtn = wrapper.findAll('.settings__link').find((b) => b.text() === '修改');
    await editBtn!.trigger('click');
    await flushPromises();

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await flushPromises();
    expect(wrapper.text()).not.toContain('按下新组合');
  });

  it('单项恢复默认只重置该项', async () => {
    const custom: AppSettings = {
      ...settings,
      shortcuts: { ...DEFAULT_SHORTCUTS, new_memo: 'Cmd+Alt+N' },
    };
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings: custom } });
    const resetBtns = wrapper.findAll('.settings__link').filter((b) => b.text() === '恢复默认');
    await resetBtns[0].trigger('click');
    await flushPromises();

    const patch = wrapper.emitted('update')![0][0] as { shortcuts: AppSettings['shortcuts'] };
    expect(patch.shortcuts.new_memo).toBe('Cmd+N');
    expect(patch.shortcuts.save).toBe('Cmd+S');
  });

  it('全部恢复默认重置整套绑定', async () => {
    const custom: AppSettings = {
      ...settings,
      shortcuts: { ...DEFAULT_SHORTCUTS, new_memo: 'Cmd+Alt+N' },
    };
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings: custom } });
    const allBtn = wrapper.findAll('button').find((b) => b.text().includes('全部恢复默认'));
    await allBtn!.trigger('click');
    await flushPromises();

    expect(wrapper.emitted('update')).toEqual([[{ shortcuts: DEFAULT_SHORTCUTS }]]);
  });

  it('Esc 关闭速记窗项为固定不可改', () => {
    const wrapper = mount(SettingsPanel, { props: { visible: true, settings } });
    expect(wrapper.text()).toContain('关闭速记窗');
    expect(wrapper.text()).toContain('固定，不可修改');
  });
});