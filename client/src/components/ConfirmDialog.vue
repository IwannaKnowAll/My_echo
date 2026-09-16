<script setup lang="ts">
import { onBeforeUnmount, watch } from 'vue';
import AppButton from './AppButton.vue';

const props = withDefaults(
  defineProps<{
    visible: boolean;
    title?: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    loading?: boolean;
  }>(),
  {
    title: '删除备忘录',
    confirmText: '删除',
    cancelText: '取消',
    loading: false,
  },
);

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

function onConfirm(): void {
  if (props.loading) {
    return;
  }
  emit('confirm');
}

function onCancel(): void {
  if (props.loading) {
    return;
  }
  emit('cancel');
}

function onBackdropClick(): void {
  onCancel();
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    onCancel();
  }
}

watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      window.addEventListener('keydown', onKeydown);
    } else {
      window.removeEventListener('keydown', onKeydown);
    }
  },
);

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <div v-if="visible" class="dialog-backdrop" @click="onBackdropClick">
    <div class="dialog" role="dialog" aria-modal="true" @click.stop>
      <h2 class="dialog__title">{{ title }}</h2>
      <p class="dialog__message">{{ message }}</p>
      <div class="dialog__actions">
        <AppButton :label="cancelText" variant="secondary" @click="onCancel" />
        <AppButton
          :label="confirmText"
          variant="danger"
          :loading="loading"
          @click="onConfirm"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog {
  width: 360px;
  max-width: calc(100% - var(--space-8));
  box-sizing: border-box;
  background-color: var(--color-surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  padding: var(--space-6);
}

.dialog__title {
  margin: 0 0 var(--space-3);
  color: var(--color-text);
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  line-height: var(--line-height-tight);
}

.dialog__message {
  margin: 0 0 var(--space-6);
  color: var(--color-text-secondary);
  font-size: var(--font-size-md);
  line-height: var(--line-height-normal);
}

.dialog__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
}
</style>