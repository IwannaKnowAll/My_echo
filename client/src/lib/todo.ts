// 待办行解析与切换：统一服务于 TextArea 原文与 checkbox 渲染两层（契约 2.6）。

import type { TodoPart } from '../types/memo';

/** 匹配 `- [ ]` / `- [x]` 待办行（允许前导空格，忽略大小写 x） */
const TODO_LINE_RE = /^[ \t]*-[ \t]*\[([ xX])\][ \t]?(.*)$/;

/** 解析单行，非待办行返回 null */
function parseTodoLine(line: string): TodoPart | null {
  const match = TODO_LINE_RE.exec(line);
  if (match === null) {
    return null;
  }
  return {
    line,
    checked: match[1] !== ' ',
    text: match[2] ?? '',
  };
}

/** 解析正文全部待办行（按行序） */
export function parseTodoLines(content: string): TodoPart[] {
  const parts: TodoPart[] = [];
  for (const line of content.split('\n')) {
    const part = parseTodoLine(line);
    if (part !== null) {
      parts.push(part);
    }
  }
  return parts;
}

/** 统计已勾选数与总数 */
export function countTodo(content: string): { done: number; total: number } {
  const parts = parseTodoLines(content);
  const total = parts.length;
  const done = parts.filter((part) => part.checked).length;
  return { done, total };
}

/**
 * 切换第 index 个待办行的完成状态（[ ] ↔ [x]），返回新的正文文本。
 * 若 index 越界则原样返回。
 */
export function toggleTodoLine(content: string, index: number): string {
  const lines = content.split('\n');
  let seen = 0;
  for (let i = 0; i < lines.length; i += 1) {
    const part = parseTodoLine(lines[i]);
    if (part === null) {
      continue;
    }
    if (seen === index) {
      const marker = part.checked ? '[ ]' : '[x]';
      lines[i] = lines[i].replace(/\[([ xX])\]/, marker);
      return lines.join('\n');
    }
    seen += 1;
  }
  return content;
}