<script setup lang="ts">
import { ref, watch } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    multiline?: boolean;
    maxlength?: number;
    disabled?: boolean;
    error?: string;
    autofocus?: boolean;
  }>(),
  {
    placeholder: '',
    multiline: false,
    maxlength: undefined,
    disabled: false,
    error: '',
    autofocus: false,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
  blur: [];
  focus: [];
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const textareaRef = ref<HTMLTextAreaElement | null>(null);

function onInput(event: Event): void {
  const target = event.target as HTMLInputElement | HTMLTextAreaElement;
  emit('update:modelValue', target.value);
}

function onBlur(): void {
  emit('blur');
}

function onFocus(): void {
  emit('focus');
}

watch(
  () => props.autofocus,
  (shouldFocus) => {
    if (!shouldFocus) {
      return;
    }
    if (props.multiline) {
      textareaRef.value?.focus();
    } else {
      inputRef.value?.focus();
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="field">
    <textarea
      v-if="multiline"
      ref="textareaRef"
      class="control control--textarea"
      :class="{ 'control--error': error }"
      :value="modelValue"
      :placeholder="placeholder"
      :maxlength="maxlength"
      :disabled="disabled"
      @input="onInput"
      @blur="onBlur"
      @focus="onFocus"
    />
    <input
      v-else
      ref="inputRef"
      class="control"
      :class="{ 'control--error': error }"
      type="text"
      :value="modelValue"
      :placeholder="placeholder"
      :maxlength="maxlength"
      :disabled="disabled"
      @input="onInput"
      @blur="onBlur"
      @focus="onFocus"
    />
    <p v-if="error" class="field__error">{{ error }}</p>
  </div>
</template>

<style scoped>
.field {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.control {
  width: 100%;
  box-sizing: border-box;
  background-color: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
  font-family: var(--font-family);
  font-size: var(--font-size-md);
  line-height: var(--line-height-normal);
  outline: none;
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
}

.control--textarea {
  resize: none;
}

.control:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
}

.control:disabled {
  opacity: 0.5;
}

.control--error {
  border-color: var(--color-danger);
}

.field__error {
  margin: 0;
  color: var(--color-danger);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
}
</style>