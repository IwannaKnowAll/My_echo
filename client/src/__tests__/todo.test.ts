import { describe, expect, it } from 'vitest';
import { countTodo, parseTodoLines, toggleTodoLine } from '../lib/todo';

describe('parseTodoLines 解析边界', () => {
  it('无待办内容返回空数组', () => {
    expect(parseTodoLines('')).toEqual([]);
    expect(parseTodoLines('普通文本\n没有待办行')).toEqual([]);
    expect(parseTodoLines('- 这是列表但无方括号')).toEqual([]);
  });

  it('混合行只解析待办行并保留行序', () => {
    const parts = parseTodoLines('标题\n- [ ] 第一项\n说明\n- [x] 第二项');
    expect(parts).toHaveLength(2);
    expect(parts[0].line).toBe('- [ ] 第一项');
    expect(parts[0].checked).toBe(false);
    expect(parts[0].text).toBe('第一项');
    expect(parts[1].checked).toBe(true);
    expect(parts[1].text).toBe('第二项');
  });

  it('识别前导空格与大小写 x', () => {
    const parts = parseTodoLines('  - [X] 大写\n\t- [x] 制表符前导');
    expect(parts).toHaveLength(2);
    expect(parts[0].checked).toBe(true);
    expect(parts[1].checked).toBe(true);
  });

  it('全角空格在方括号内不解析', () => {
    // 方括号内为全角空格（U+3000），非 - [ ] 半角语法，应为纯文本。
    expect(parseTodoLines('- [　] 全角括号内')).toHaveLength(0);
  });

  it('全角空格作为 - 与 [ 之间的分隔符不解析', () => {
    // PRD 3.2：半角与全角空格不视为待办语法。全角空格（U+3000）分隔的 -　[ ] 不应解析为待办。
    expect(parseTodoLines('-　[ ] 全角分隔')).toHaveLength(0);
  });
});

describe('countTodo 统计', () => {
  it('无待办返回 0/0', () => {
    expect(countTodo('普通文字')).toEqual({ done: 0, total: 0 });
  });

  it('统计已完成/总数', () => {
    expect(countTodo('- [ ] a\n- [x] b\n- [x] c')).toEqual({ done: 2, total: 3 });
  });
});

describe('toggleTodoLine 切换', () => {
  it('[ ] 切换为 [x]', () => {
    const content = '- [ ] 待办';
    expect(toggleTodoLine(content, 0)).toBe('- [x] 待办');
  });

  it('[x] 切换回 [ ]', () => {
    expect(toggleTodoLine('- [x] 待办', 0)).toBe('- [ ] 待办');
  });

  it('只切换目标行，其余行不动', () => {
    const content = '- [ ] 第一条\n普通文本\n- [x] 第三条';
    expect(toggleTodoLine(content, 1)).toBe('- [ ] 第一条\n普通文本\n- [ ] 第三条');
  });

  it('index 越界原样返回', () => {
    const content = '- [ ] 唯一待办';
    expect(toggleTodoLine(content, 5)).toBe(content);
    expect(toggleTodoLine(content, -1)).toBe(content);
  });
});