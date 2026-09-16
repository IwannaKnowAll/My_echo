import { describe, expect, it } from 'vitest';
import { comboFromEvent, DEFAULT_SHORTCUTS, matchesCombo } from '../lib/shortcuts';

describe('DEFAULT_SHORTCUTS 默认绑定', () => {
  it('与 AGENTS 5.4 默认绑定协议一致', () => {
    expect(DEFAULT_SHORTCUTS.new_memo).toBe('Cmd+N');
    expect(DEFAULT_SHORTCUTS.save).toBe('Cmd+S');
    expect(DEFAULT_SHORTCUTS.delete).toBe('Cmd+Backspace');
    expect(DEFAULT_SHORTCUTS.focus_search).toBe('Cmd+F');
    expect(DEFAULT_SHORTCUTS.quick_note).toBe('Cmd+Shift+Space');
  });

  it('删除键字符串为 Cmd+Backspace（macOS 退格）', () => {
    expect(DEFAULT_SHORTCUTS.delete).toBe('Cmd+Backspace');
  });
});

describe('comboFromEvent 组合解析', () => {
  function key(name: string, init: KeyboardEventInit = {}): KeyboardEvent {
    return new KeyboardEvent('keydown', { key: name, ...init });
  }

  it('Cmd+N 生成 Cmd+N', () => {
    expect(comboFromEvent(key('n', { metaKey: true }))).toBe('Cmd+N');
  });

  it('修饰键序固定 Cmd/Option/Control/Shift', () => {
    const e = key('s', { metaKey: true, altKey: true, ctrlKey: true, shiftKey: true });
    expect(comboFromEvent(e)).toBe('Cmd+Option+Control+Shift+S');
  });

  it('空格主键规范为 Space', () => {
    expect(comboFromEvent(key(' ', { metaKey: true, shiftKey: true }))).toBe('Cmd+Shift+Space');
  });

  it('Backspace 主键保留原名', () => {
    expect(comboFromEvent(key('Backspace', { metaKey: true }))).toBe('Cmd+Backspace');
  });

  it('箭头键规范为 Up/Down', () => {
    expect(comboFromEvent(key('ArrowUp', { metaKey: true }))).toBe('Cmd+Up');
  });

  it('纯修饰键按下返回 null', () => {
    expect(comboFromEvent(key('Meta', { metaKey: true }))).toBeNull();
    expect(comboFromEvent(key('Shift', { shiftKey: true }))).toBeNull();
  });
});

describe('matchesCombo 匹配', () => {
  it('命中组合返回 true', () => {
    const e = new KeyboardEvent('keydown', { key: 'n', metaKey: true });
    expect(matchesCombo(e, 'Cmd+N')).toBe(true);
  });

  it('组合不一致返回 false', () => {
    const e = new KeyboardEvent('keydown', { key: 'f', metaKey: true });
    expect(matchesCombo(e, 'Cmd+N')).toBe(false);
  });
});