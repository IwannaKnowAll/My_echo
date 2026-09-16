import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import MemoListItem from '../components/MemoListItem.vue';
import type { Memo } from '../types/memo';

const memo: Memo = {
  id: 1,
  title: '购物清单',
  content: '牛奶、鸡蛋',
  created_at: '2026-09-14T03:15:00.000Z',
  updated_at: '2026-09-14T03:15:00.000Z',
  remind_at: '',
};

describe('MemoListItem', () => {
  it('渲染标题、时间与正文摘要', () => {
    const wrapper = mount(MemoListItem, {
      props: { memo, timeText: '2026-09-14 11:15' },
    });
    expect(wrapper.get('.item__title').text()).toBe('购物清单');
    expect(wrapper.get('.item__time').text()).toBe('2026-09-14 11:15');
    expect(wrapper.get('.item__summary').text()).toBe('牛奶、鸡蛋');
  });

  it('content 为空时不渲染摘要行', () => {
    const wrapper = mount(MemoListItem, {
      props: { memo: { ...memo, content: '' }, timeText: 'x' },
    });
    expect(wrapper.find('.item__summary').exists()).toBe(false);
  });

  it('点击触发 click 事件', async () => {
    const wrapper = mount(MemoListItem, {
      props: { memo, timeText: 'x' },
    });
    await wrapper.trigger('click');
    expect(wrapper.emitted('click')).toHaveLength(1);
  });

  it('外层为 div 且带 role=button 与键盘可达', () => {
    const wrapper = mount(MemoListItem, {
      props: { memo, timeText: 'x' },
    });
    const root = wrapper.get('.item');
    expect(root.element.tagName).toBe('DIV');
    expect(root.attributes('role')).toBe('button');
    expect(root.attributes('tabindex')).toBe('0');
  });

  it('按 Enter 触发 click 事件（键盘可达行为）', async () => {
    const wrapper = mount(MemoListItem, {
      props: { memo, timeText: 'x' },
    });
    await wrapper.get('.item').trigger('keydown', { key: 'Enter' });
    expect(wrapper.emitted('click')).toHaveLength(1);
  });
});