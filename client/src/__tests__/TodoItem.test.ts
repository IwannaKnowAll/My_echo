import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import TodoItem from '../components/TodoItem.vue';

describe('TodoItem', () => {
  it('渲染待办文本', () => {
    const wrapper = mount(TodoItem, {
      props: { line: '- [ ] abc', checked: false, text: 'abc' },
    });
    expect(wrapper.get('.todo__text').text()).toBe('abc');
  });

  it('checked 时加勾选样式并渲染对勾', () => {
    const wrapper = mount(TodoItem, {
      props: { line: '- [x] abc', checked: true, text: 'abc' },
    });
    expect(wrapper.get('.todo').classes()).toContain('todo--checked');
    expect(wrapper.find('.todo__check').exists()).toBe(true);
  });

  it('点击切换触发 toggle 且传反值', async () => {
    const wrapper = mount(TodoItem, {
      props: { line: '- [ ] abc', checked: false, text: 'abc' },
    });
    await wrapper.get('.todo').trigger('click');
    expect(wrapper.emitted('toggle')).toEqual([[true]]);
  });

  it('disabled 时点击不触发 toggle', async () => {
    const wrapper = mount(TodoItem, {
      props: { line: '- [ ] abc', checked: false, text: 'abc', disabled: true },
    });
    await wrapper.get('.todo').trigger('click');
    expect(wrapper.emitted('toggle')).toBeUndefined();
  });
});