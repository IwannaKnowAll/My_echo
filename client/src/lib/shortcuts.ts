// 快捷键记法与解析：前端 keydown 监听 + 录制共用同一套组合字符串生成规则。

import type { ShortcutBindings } from '../types/memo';

/** 默认绑定（前端约定，须与壳层 settings 默认值保持一致） */
export const DEFAULT_SHORTCUTS: ShortcutBindings = {
  new_memo: 'Cmd+N',
  save: 'Cmd+S',
  delete: 'Cmd+Backspace',
  focus_search: 'Cmd+F',
  quick_note: 'Cmd+Shift+Space',
};

/** 纯修饰键按下时不构成组合，等待主键 */
const MODIFIER_KEYS = new Set(['Meta', 'Alt', 'Control', 'Shift']);

/** 主键展示名规范化 */
function formatMainKey(key: string): string {
  if (key === ' ') {
    return 'Space';
  }
  if (key === 'ArrowUp') {
    return 'Up';
  }
  if (key === 'ArrowDown') {
    return 'Down';
  }
  if (key === 'ArrowLeft') {
    return 'Left';
  }
  if (key === 'ArrowRight') {
    return 'Right';
  }
  if (key.length === 1) {
    return key.toUpperCase();
  }
  return key;
}

/**
 * 从键盘事件生成组合字符串，修饰键序固定为 Cmd/Option/Control/Shift + 主键。
 * 纯修饰键按下返回 null（等待完整组合）。
 */
export function comboFromEvent(event: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(event.key)) {
    return null;
  }
  const mainKey = formatMainKey(event.key);
  const parts: string[] = [];
  if (event.metaKey) {
    parts.push('Cmd');
  }
  if (event.altKey) {
    parts.push('Option');
  }
  if (event.ctrlKey) {
    parts.push('Control');
  }
  if (event.shiftKey) {
    parts.push('Shift');
  }
  parts.push(mainKey);
  return parts.join('+');
}

/** 判断按键事件是否命中指定组合 */
export function matchesCombo(event: KeyboardEvent, combo: string): boolean {
  const generated = comboFromEvent(event);
  return generated !== null && generated === combo;
}