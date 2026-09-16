<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
  }>(),
  {
    placeholder: '搜索标题或内容',
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
  clear: [];
}>();

const hasValue = computed(() => props.modelValue.length > 0);

function onInput(event: Event): void {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', target.value);
}

function onClear(): void {
  emit('clear');
}
</script>

<template>
  <div class="search">
    <span class="search__icon" aria-hidden="true">🔍</span>
    <input
      class="search__input"
      type="text"
      :value="modelValue"
      :placeholder="placeholder"
      @input="onInput"
    />
    <button
      v-if="hasValue"
      type="button"
      class="search__clear"
      aria-label="清空搜索"
      @click="onClear"
    >
      ×
    </button>
  </div>
</template>

<style scoped>
.search {
  position: relative;
  display: flex;
  align-items: center;
}

.search__icon {
  position: absolute;
  left: var(--space-3);
  font-size: var(--font-size-sm);
  pointer-events: none;
}

.search__input {
  width: 100%;
  box-sizing: border-box;
  background-color: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-6) var(--space-2) var(--space-8);
  font-family: var(--font-family);
  font-size: var(--font-size-md);
  line-height: var(--line-height-normal);
  outline: none;
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
}

.search__input:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
}

.search__clear {
  position: absolute;
  right: var(--space-2);
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: var(--radius-full);
  background-color: var(--color-surface-hover);
  color: var(--color-text-secondary);
  font-size: var(--font-size-lg);
  line-height: 1;
  cursor: pointer;
}

.search__clear:hover {
  color: var(--color-text);
}
</style>