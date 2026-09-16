<script setup lang="ts">
withDefaults(
  defineProps<{
    checked: boolean;
    text: string;
    disabled?: boolean;
  }>(),
  {
    disabled: false,
  },
);

defineEmits<{
  toggle: [checked: boolean];
}>();
</script>

<template>
  <button
    type="button"
    class="todo"
    :class="{ 'todo--checked': checked }"
    :disabled="disabled"
    @click="$emit('toggle', !checked)"
  >
    <span class="todo__box" :class="{ 'todo__box--checked': checked }" aria-hidden="true">
      <span v-if="checked" class="todo__check">✓</span>
    </span>
    <span class="todo__text" :class="{ 'todo__text--checked': checked }">{{ text }}</span>
  </button>
</template>

<style scoped>
.todo {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  border: none;
  background: transparent;
  padding: var(--space-1) 0;
  text-align: left;
  cursor: pointer;
  font-family: var(--font-family);
}

.todo:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.todo__box {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--color-checkbox-border);
  border-radius: var(--radius-sm);
  background-color: transparent;
}

.todo__box--checked {
  background-color: var(--color-checkbox-checked);
  border-color: var(--color-checkbox-checked);
}

.todo__check {
  color: var(--color-checkbox-check);
  font-size: var(--font-size-xs);
  line-height: 1;
}

.todo__text {
  color: var(--color-text);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-normal);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.todo__text--checked {
  color: var(--color-text-tertiary);
  text-decoration: line-through;
}
</style>