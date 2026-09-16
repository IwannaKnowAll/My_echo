import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import SearchBox from '../components/SearchBox.vue';

describe('SearchBox', () => {
  it('渲染默认 placeholder', () => {
    const wrapper = mount(SearchBox, { props: { modelValue: '' } });
    expect(wrapper.get('input').attributes('placeholder')).toBe('搜索标题或内容');
  });

  it('输入触发 update:modelValue', async () => {
    const wrapper = mount(SearchBox, { props: { modelValue: '' } });
    await wrapper.get('input').setValue('关键词');
    expect(wrapper.emitted('update:modelValue')).toEqual([['关键词']]);
  });

  it('非空时显示清空按钮，点击仅触发 clear（不再重复触发 update:modelValue）', async () => {
    const wrapper = mount(SearchBox, { props: { modelValue: '关键词' } });
    expect(wrapper.find('.search__clear').exists()).toBe(true);

    await wrapper.get('.search__clear').trigger('click');
    expect(wrapper.emitted('clear')).toHaveLength(1);
    expect(wrapper.emitted('update:modelValue')).toBeUndefined();
  });

  it('空输入时不显示清空按钮', () => {
    const wrapper = mount(SearchBox, { props: { modelValue: '' } });
    expect(wrapper.find('.search__clear').exists()).toBe(false);
  });
});