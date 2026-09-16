// 主题管理器：读 get_settings 解析三态，统一落实到 html[data-theme]，system 用 matchMedia 实时跟随。

import { getSettings } from './api';
import type { AppSettings, ThemeMode } from '../types/memo';

const DARK_SCHEME_QUERY = '(prefers-color-scheme: dark)';

let currentMode: ThemeMode = 'system';
let systemListenerAttached = false;

/** 当前系统是否偏好深色；无 matchMedia 环境（如测试）默认浅色 */
function systemPrefersDark(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false;
  }
  return window.matchMedia(DARK_SCHEME_QUERY).matches;
}

/** 将三态解析为具体 light/dark */
export function resolveTheme(mode: ThemeMode): 'light' | 'dark' {
  if (mode === 'system') {
    return systemPrefersDark() ? 'dark' : 'light';
  }
  return mode;
}

function applyTheme(mode: ThemeMode): void {
  const resolved = resolveTheme(mode);
  document.documentElement.setAttribute('data-theme', resolved);
}

function handleSystemChange(): void {
  if (currentMode === 'system') {
    applyTheme('system');
  }
}

function attachSystemListener(): void {
  if (systemListenerAttached) {
    return;
  }
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return;
  }
  const mediaQuery = window.matchMedia(DARK_SCHEME_QUERY);
  mediaQuery.addEventListener('change', handleSystemChange);
  systemListenerAttached = true;
}

/** 设置主题模式并即时生效；system 态会实时跟随系统外观变化 */
export function setThemeMode(mode: ThemeMode): void {
  currentMode = mode;
  attachSystemListener();
  applyTheme(mode);
}

/** 读配置并应用主题；读取失败回退 system。返回完整配置供上层复用。 */
export async function initTheme(): Promise<AppSettings | null> {
  try {
    const settings = await getSettings();
    const mode: ThemeMode = settings.theme ? settings.theme : 'system';
    setThemeMode(mode);
    return settings;
  } catch {
    setThemeMode('system');
    return null;
  }
}