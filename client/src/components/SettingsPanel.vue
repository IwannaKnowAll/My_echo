<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue';
import type {
  AppSettings,
  CloseBehavior,
  ShortcutBindings,
  ThemeMode,
  UpdateSettingsInput,
} from '../types/memo';
import { comboFromEvent, DEFAULT_SHORTCUTS } from '../lib/shortcuts';
import AppButton from './AppButton.vue';

const props = withDefaults(
  defineProps<{
    visible: boolean;
    settings: AppSettings | null;
    error?: string;
  }>(),
  {
    error: '',
  },
);

const emit = defineEmits<{
  update: [patch: UpdateSettingsInput];
  close: [];
}>();

interface ShortcutItem {
  key: keyof ShortcutBindings;
  label: string;
}

const SHORTCUT_ITEMS: ShortcutItem[] = [
  { key: 'new_memo', label: '新建备忘录' },
  { key: 'save', label: '保存' },
  { key: 'delete', label: '删除' },
  { key: 'focus_search', label: '聚焦搜索' },
  { key: 'quick_note', label: '新建速记' },
];

const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
  { value: 'system', label: '跟随系统' },
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
];

const recordingKey = ref<keyof ShortcutBindings | null>(null);
const recordError = ref('');

function cloneBindings(): ShortcutBindings {
  const s = props.settings?.shortcuts ?? DEFAULT_SHORTCUTS;
  return {
    new_memo: s.new_memo,
    save: s.save,
    delete: s.delete,
    focus_search: s.focus_search,
    quick_note: s.quick_note,
  };
}

function comboOf(key: keyof ShortcutBindings): string {
  return props.settings?.shortcuts[key] ?? '';
}

function selectTheme(mode: ThemeMode): void {
  emit('update', { theme: mode });
}

function toggleCloseBehavior(): void {
  const behavior: CloseBehavior =
    props.settings?.close_behavior === 'hide' ? 'quit' : 'hide';
  emit('update', { close_behavior: behavior });
}

function startRecord(key: keyof ShortcutBindings): void {
  recordingKey.value = key;
  recordError.value = '';
}

function isValidCombo(combo: string): boolean {
  if (/^F(?:[1-9]|1[0-2])$/.test(combo)) {
    return true;
  }
  return /(?:Cmd|Option|Control|Shift)\+/.test(combo);
}

function applyCombo(combo: string): void {
  const key = recordingKey.value;
  if (key === null) {
    return;
  }
  recordingKey.value = null;
  const next = cloneBindings();
  next[key] = combo;
  emit('update', { shortcuts: next });
}

function resetItem(key: keyof ShortcutBindings): void {
  const next = cloneBindings();
  next[key] = DEFAULT_SHORTCUTS[key];
  emit('update', { shortcuts: next });
}

function resetAll(): void {
  emit('update', { shortcuts: DEFAULT_SHORTCUTS });
}

function onKeydown(event: KeyboardEvent): void {
  if (recordingKey.value === null) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  if (event.key === 'Escape') {
    recordingKey.value = null;
    recordError.value = '';
    return;
  }
  const combo = comboFromEvent(event);
  if (combo === null) {
    return;
  }
  if (!isValidCombo(combo)) {
    recordError.value = '快捷键必须包含修饰键或为功能键';
    return;
  }
  recordError.value = '';
  applyCombo(combo);
}

watch(
  () => recordingKey.value,
  (key) => {
    if (key !== null) {
      window.addEventListener('keydown', onKeydown, true);
    } else {
      window.removeEventListener('keydown', onKeydown, true);
    }
  },
);

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown, true);
});
</script>

<template>
  <div v-if="visible" class="settings-backdrop" @click.self="emit('close')">
    <div class="settings" role="dialog" aria-modal="true">
      <header class="settings__header">
        <h2 class="settings__title">设置</h2>
        <button type="button" class="settings__close" aria-label="关闭设置" @click="emit('close')">
          ×
        </button>
      </header>

      <div v-if="!settings" class="settings__loading">加载中…</div>
      <template v-else>
        <p v-if="error" class="settings__error">{{ error }}</p>

        <section class="settings__section">
          <h3 class="settings__label">主题</h3>
          <div class="settings__segments">
            <button
              v-for="option in THEME_OPTIONS"
              :key="option.value"
              type="button"
              class="settings__segment"
              :class="{ 'settings__segment--active': settings.theme === option.value }"
              @click="selectTheme(option.value)"
            >
              {{ option.label }}
            </button>
          </div>
        </section>

        <section class="settings__section">
          <div class="settings__row">
            <div class="settings__row-text">
              <h3 class="settings__label">关闭主窗口后保留在菜单栏</h3>
              <p class="settings__hint">开启后关闭窗口仅隐藏，从菜单栏图标退出才完全退出</p>
            </div>
            <button
              type="button"
              class="switch"
              :class="{ 'switch--on': settings.close_behavior === 'hide' }"
              role="switch"
              :aria-checked="settings.close_behavior === 'hide'"
              @click="toggleCloseBehavior"
            >
              <span class="switch__thumb" />
            </button>
          </div>
        </section>

        <section class="settings__section">
          <h3 class="settings__label">快捷键</h3>
          <p v-if="recordError" class="settings__record-error">{{ recordError }}</p>
          <ul class="settings__list">
            <li
              v-for="item in SHORTCUT_ITEMS"
              :key="item.key"
              class="settings__item"
            >
              <span class="settings__item-label">{{ item.label }}</span>
              <span v-if="recordingKey === item.key" class="settings__recording">按下新组合</span>
              <span v-else class="settings__kbd">{{ comboOf(item.key) }}</span>
              <button type="button" class="settings__link" @click="startRecord(item.key)">
                修改
              </button>
              <button type="button" class="settings__link" @click="resetItem(item.key)">
                恢复默认
              </button>
            </li>
            <li class="settings__item settings__item--fixed">
              <span class="settings__item-label">关闭速记窗</span>
              <span class="settings__kbd">Esc</span>
              <span class="settings__fixed-hint">固定，不可修改</span>
            </li>
          </ul>
          <div class="settings__footer">
            <AppButton label="全部恢复默认" variant="secondary" @click="resetAll" />
          </div>
        </section>
      </template>
    </div>
  </div>
</template>

<style scoped>
.settings-backdrop {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.settings {
  width: 440px;
  max-width: calc(100% - var(--space-8));
  max-height: calc(100% - var(--space-8));
  overflow-y: auto;
  box-sizing: border-box;
  background-color: var(--color-surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  padding: var(--space-6);
}

.settings__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-4);
}

.settings__title {
  margin: 0;
  color: var(--color-text);
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
}

.settings__close {
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-lg);
  line-height: 1;
  cursor: pointer;
}

.settings__close:hover {
  color: var(--color-text);
}

.settings__loading {
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  padding: var(--space-4) 0;
}

.settings__error {
  margin: 0 0 var(--space-3);
  color: var(--color-danger);
  font-size: var(--font-size-sm);
}

.settings__section {
  margin-bottom: var(--space-5);
}

.settings__label {
  margin: 0 0 var(--space-2);
  color: var(--color-text);
  font-size: var(--font-size-md);
  font-weight: var(--font-weight-medium);
}

.settings__segments {
  display: inline-flex;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.settings__segment {
  border: none;
  background-color: var(--color-surface);
  color: var(--color-text-secondary);
  padding: var(--space-2) var(--space-4);
  font-size: var(--font-size-sm);
  cursor: pointer;
}

.settings__segment + .settings__segment {
  border-left: 1px solid var(--color-border);
}

.settings__segment--active {
  background-color: var(--color-accent);
  color: var(--color-on-accent);
}

.settings__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
}

.settings__row-text {
  flex: 1;
}

.settings__hint {
  margin: 0;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
  line-height: var(--line-height-normal);
}

.switch {
  flex-shrink: 0;
  position: relative;
  width: 44px;
  height: 26px;
  border: none;
  border-radius: var(--radius-full);
  background-color: var(--color-switch-off);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.switch--on {
  background-color: var(--color-switch-on);
}

.switch__thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background-color: var(--color-switch-thumb);
  transition: left 0.15s ease;
}

.switch--on .switch__thumb {
  left: 20px;
}

.settings__list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.settings__item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-height: 28px;
}

.settings__item-label {
  flex: 1;
  color: var(--color-text);
  font-size: var(--font-size-sm);
}

.settings__item--fixed .settings__item-label,
.settings__item--fixed .settings__kbd {
  color: var(--color-text-tertiary);
}

.settings__kbd {
  min-width: 96px;
  text-align: center;
  background-color: var(--color-surface-hover);
  color: var(--color-text-secondary);
  border-radius: var(--radius-sm);
  padding: var(--space-1) var(--space-2);
  font-size: var(--font-size-sm);
  font-family: var(--font-family);
}

.settings__recording {
  min-width: 96px;
  text-align: center;
  color: var(--color-accent);
  font-size: var(--font-size-sm);
}

.settings__link {
  border: none;
  background: transparent;
  color: var(--color-accent);
  font-size: var(--font-size-sm);
  cursor: pointer;
  padding: 0;
}

.settings__link:hover {
  text-decoration: underline;
}

.settings__fixed-hint {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
}

.settings__record-error {
  margin: 0 0 var(--space-2);
  color: var(--color-danger);
  font-size: var(--font-size-sm);
}

.settings__footer {
  margin-top: var(--space-3);
  display: flex;
  justify-content: flex-end;
}
</style>