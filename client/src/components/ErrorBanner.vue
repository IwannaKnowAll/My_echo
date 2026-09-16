<script setup lang="ts">
defineProps<{
  message: string;
}>();

defineEmits<{
  close: [];
}>();
</script>

<template>
  <div class="banner" role="alert">
    <span class="banner__message">{{ message }}</span>
    <button type="button" class="banner__close" aria-label="关闭提示" @click="$emit('close')">
      ×
    </button>
  </div>
</template>

<style scoped>
.banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  background-color: var(--color-surface);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
}

.banner__message {
  color: var(--color-danger);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-normal);
}

.banner__close {
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-md);
  line-height: 1;
  cursor: pointer;
}

.banner__close:hover {
  color: var(--color-text);
}
</style>