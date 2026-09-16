<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { createMemo, emitQuickNoteSaved } from '../lib/api';
import ConfirmDialog from './ConfirmDialog.vue';

const title = ref('');
const content = ref('');
const saving = ref(false);
const titleError = ref('');
const formError = ref('');
const confirmVisible = ref(false);

const dirty = computed(() => title.value.trim() !== '' || content.value !== '');

async function save(): Promise<void> {
  if (saving.value) {
    return;
  }
  const trimmed = title.value.trim();
  if (trimmed === '') {
    titleError.value = '标题不能为空';
    return;
  }
  if ([...title.value].length > 100) {
    titleError.value = '标题不能超过 100 个字符';
    return;
  }
  titleError.value = '';
  formError.value = '';
  saving.value = true;
  try {
    await createMemo({ title: title.value, content: content.value });
    await emitQuickNoteSaved();
    await getCurrentWindow().close();
  } catch {
    formError.value = '操作失败，请重试';
  } finally {
    saving.value = false;
  }
}

function closeWindow(): void {
  void getCurrentWindow().close();
}

function requestClose(): void {
  if (dirty.value) {
    confirmVisible.value = true;
  } else {
    closeWindow();
  }
}

function discard(): void {
  confirmVisible.value = false;
  closeWindow();
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && !confirmVisible.value) {
    event.preventDefault();
    requestClose();
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <div class="quicknote">
    <header class="quicknote__header">
      <span class="quicknote__title">速记</span>
      <button type="button" class="quicknote__close" aria-label="关闭" @click="requestClose">×</button>
    </header>
    <div v-if="formError" class="quicknote__error">{{ formError }}</div>
    <div class="quicknote__field">
      <input
        v-model="title"
        class="quicknote__title-input"
        :class="{ 'quicknote__title-input--error': titleError }"
        type="text"
        placeholder="标题"
        autofocus
        :disabled="saving"
        @keydown.enter.prevent="save"
      />
      <p v-if="titleError" class="quicknote__title-error">{{ titleError }}</p>
    </div>
    <textarea
      v-model="content"
      class="quicknote__content"
      placeholder="正文（可选）"
      :disabled="saving"
    />
    <footer class="quicknote__footer">
      <button
        type="button"
        class="quicknote__save"
        :disabled="saving"
        @click="save"
      >
        <span v-if="saving" class="quicknote__spinner" aria-hidden="true" />
        保存
      </button>
    </footer>

    <ConfirmDialog
      :visible="confirmVisible"
      title="速记"
      message="丢弃未保存的速记？"
      confirm-text="丢弃"
      cancel-text="取消"
      @confirm="discard"
      @cancel="confirmVisible = false"
    />
  </div>
</template>

<style scoped>
.quicknote {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4);
  background-color: var(--color-quicknote-bg);
  border: 1px solid var(--color-quicknote-border);
  box-sizing: border-box;
}

.quicknote__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.quicknote__title {
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
}

.quicknote__close {
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-lg);
  line-height: 1;
  cursor: pointer;
}

.quicknote__close:hover {
  color: var(--color-text);
}

.quicknote__error {
  flex-shrink: 0;
  color: var(--color-danger);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
}

.quicknote__field {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.quicknote__title-input {
  box-sizing: border-box;
  width: 100%;
  background-color: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
  font-family: var(--font-family);
  font-size: var(--font-size-lg);
  line-height: var(--line-height-tight);
  outline: none;
}

.quicknote__title-input:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
}

.quicknote__title-input--error {
  border-color: var(--color-danger);
}

.quicknote__title-error {
  margin: 0;
  color: var(--color-danger);
  font-size: var(--font-size-sm);
}

.quicknote__content {
  flex: 1;
  min-height: 120px;
  box-sizing: border-box;
  width: 100%;
  background-color: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
  font-family: var(--font-family);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-normal);
  outline: none;
  resize: none;
}

.quicknote__content:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
}

.quicknote__footer {
  flex-shrink: 0;
  display: flex;
  justify-content: flex-end;
}

.quicknote__save {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  background-color: var(--color-accent);
  color: var(--color-on-accent);
  border: none;
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  font-family: var(--font-family);
  font-size: var(--font-size-md);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
}

.quicknote__save:hover:not(:disabled) {
  background-color: var(--color-accent-hover);
}

.quicknote__save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.quicknote__spinner {
  width: 12px;
  height: 12px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: quicknote-spin 0.7s linear infinite;
}

@keyframes quicknote-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>