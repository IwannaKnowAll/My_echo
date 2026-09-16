import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { getSettings } from '../lib/api';
import { initTheme, resolveTheme, setThemeMode } from '../lib/theme';
import type { AppSettings } from '../types/memo';

vi.mock('../lib/api', () => ({ getSettings: vi.fn() }));

const mockGetSettings = vi.mocked(getSettings);

function makeSettings(theme: AppSettings['theme']): AppSettings {
  return {
    theme,
    close_behavior: 'quit',
    shortcuts: {
      new_memo: 'Cmd+N',
      save: 'Cmd+S',
      delete: 'Cmd+Backspace',
      focus_search: 'Cmd+F',
      quick_note: 'Cmd+Shift+Space',
    },
  };
}

function stubMatchMedia(matches: boolean): void {
  window.matchMedia = vi.fn(
    () => ({ matches, addEventListener: vi.fn() }) as unknown as MediaQueryList,
  );
}

afterEach(() => {
  document.documentElement.removeAttribute('data-theme');
  vi.unstubAllGlobals();
});

describe('resolveTheme 三态解析', () => {
  it('light/dark 直接返回自身', () => {
    expect(resolveTheme('light')).toBe('light');
    expect(resolveTheme('dark')).toBe('dark');
  });

  it('system 依据系统外观解析为 light/dark', () => {
    stubMatchMedia(true);
    expect(resolveTheme('system')).toBe('dark');

    stubMatchMedia(false);
    expect(resolveTheme('system')).toBe('light');
  });
});

describe('setThemeMode 应用', () => {
  it('三态统一落到 html[data-theme]', () => {
    stubMatchMedia(false);

    setThemeMode('light');
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');

    setThemeMode('dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');

    setThemeMode('system');
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });
});

describe('initTheme 读配置应用', () => {
  beforeEach(() => {
    mockGetSettings.mockReset();
    stubMatchMedia(false);
  });

  it('读取成功应用主题并返回完整配置', async () => {
    mockGetSettings.mockResolvedValue(makeSettings('dark'));
    const result = await initTheme();
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
    expect(result?.theme).toBe('dark');
  });

  it('读取失败回退 system', async () => {
    mockGetSettings.mockRejectedValue({ code: 'E_INTERNAL', message: 'x' });
    const result = await initTheme();
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
    expect(result).toBeNull();
  });
});