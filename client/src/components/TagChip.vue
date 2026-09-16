<script setup lang="ts">
import type { Tag } from '../types/memo';

withDefaults(
  defineProps<{
    tag: Tag;
    active?: boolean;
    removable?: boolean;
    disabled?: boolean;
  }>(),
  {
    active: false,
    removable: false,
    disabled: false,
  },
);

const emit = defineEmits<{
  click: [];
  remove: [];
}>();
</script>

<template>
  <span
    class="tag"
    :class="{ 'tag--active': active, 'tag--disabled': disabled }"
    @click="emit('click')"
  >
    <span class="tag__name">{{ tag.name }}</span>
    <button
      v-if="removable"
      type="button"
      class="tag__remove"
      :disabled="disabled"
      aria-label="移除标签"
      @click.stop="emit('remove')"
    >
      ×
    </button>
  </span>
</template>

<style scoped>
.tag {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 2px var(--space-2);
  background-color: var(--color-tag-bg);
  color: var(--color-tag-text);
  border: 1px solid var(--color-tag-border);
  border-radius: var(--radius-full);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
  white-space: nowrap;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.tag:hover {
  background-color: var(--color-tag-bg-hover);
}

.tag--active {
  background-color: var(--color-tag-active-bg);
  color: var(--color-on-accent);
  border-color: var(--color-tag-active-bg);
}

.tag--active:hover {
  background-color: var(--color-tag-active-bg);
}

.tag--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tag__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: inherit;
  font-size: var(--font-size-md);
  line-height: 1;
  padding: 0;
  cursor: pointer;
  opacity: 0.75;
}

.tag__remove:hover:not(:disabled) {
  color: var(--color-danger);
}
</style>