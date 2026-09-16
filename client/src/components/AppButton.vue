<script setup lang="ts">
defineProps<{
  label: string;
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  type?: 'button' | 'submit';
}>();

const emit = defineEmits<{
  click: [];
}>();

function handleClick(): void {
  emit('click');
}
</script>

<template>
  <button
    :type="type ?? 'button'"
    class="btn"
    :class="[`btn--${variant ?? 'secondary'}`, `btn--${size ?? 'md'}`]"
    :disabled="disabled || loading"
    @click="handleClick"
  >
    <span v-if="loading" class="btn__spinner" aria-hidden="true" />
    <span class="btn__label">{{ label }}</span>
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  font-family: var(--font-family);
  font-weight: var(--font-weight-medium);
  line-height: var(--line-height-tight);
  cursor: pointer;
  transition: background-color 0.15s ease, border-color 0.15s ease, opacity 0.15s ease;
  white-space: nowrap;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn--sm {
  font-size: var(--font-size-xs);
  padding: var(--space-1) var(--space-3);
}

.btn--md {
  font-size: var(--font-size-md);
  padding: var(--space-2) var(--space-4);
}

.btn--lg {
  font-size: var(--font-size-lg);
  padding: var(--space-3) var(--space-5);
}

.btn--primary {
  background-color: var(--color-accent);
  color: #ffffff;
}
.btn--primary:hover:not(:disabled) {
  background-color: var(--color-accent-hover);
}
.btn--primary:active:not(:disabled) {
  background-color: var(--color-accent-active);
}

.btn--danger {
  background-color: var(--color-danger);
  color: #ffffff;
}
.btn--danger:hover:not(:disabled) {
  background-color: var(--color-danger-hover);
}

.btn--secondary {
  background-color: var(--color-surface);
  color: var(--color-text);
  border-color: var(--color-border);
}
.btn--secondary:hover:not(:disabled) {
  background-color: var(--color-surface-hover);
}

.btn--ghost {
  background-color: transparent;
  color: var(--color-text);
}
.btn--ghost:hover:not(:disabled) {
  background-color: var(--color-surface-hover);
}

.btn__spinner {
  width: 12px;
  height: 12px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>