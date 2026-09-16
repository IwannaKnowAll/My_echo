<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import type { ActiveTagId, TagWithCount } from '../types/memo';
import TagChip from './TagChip.vue';

const props = withDefaults(
  defineProps<{
    tags: TagWithCount[];
    activeTagId: ActiveTagId;
    loading?: boolean;
    error?: string;
  }>(),
  {
    loading: false,
    error: '',
  },
);

const emit = defineEmits<{
  select: [tagId: number | null];
  create: [name: string];
  remove: [tag: TagWithCount];
  retry: [];
}>();

const creating = ref(false);
const newName = ref('');
const nameError = ref('');
const createInput = ref<HTMLInputElement | null>(null);
const isEmpty = computed(() => props.tags.length === 0);

function startCreate(): void {
  creating.value = true;
  newName.value = '';
  nameError.value = '';
  void nextTick(() => {
    createInput.value?.focus();
  });
}

function confirmCreate(): void {
  const name = newName.value.trim();
  if (name !== '' && [...name].length > 30) {
    nameError.value = '标签名不能超过 30 个字符';
    return;
  }
  nameError.value = '';
  creating.value = false;
  newName.value = '';
  if (name === '') {
    return;
  }
  emit('create', name);
}

function cancelCreate(): void {
  creating.value = false;
  newName.value = '';
  nameError.value = '';
}
</script>

<template>
  <div class="tags-bar">
    <button
      type="button"
      class="tags-bar__all"
      :class="{ 'tags-bar__all--active': activeTagId === null }"
      @click="emit('select', null)"
    >
      全部
    </button>
    <div class="tags-bar__list">
      <template v-if="loading">
        <span class="tags-bar__skeleton" />
        <span class="tags-bar__skeleton" />
        <span class="tags-bar__skeleton" />
      </template>
      <template v-else>
        <TagChip
          v-for="tag in tags"
          :key="tag.id"
          :tag="tag"
          :active="activeTagId === tag.id"
          removable
          @click="emit('select', tag.id)"
          @remove="emit('remove', tag)"
        />
        <span v-if="isEmpty && !creating" class="tags-bar__empty">暂无标签</span>
      </template>

      <button
        v-if="!creating"
        type="button"
        class="tags-bar__add"
        aria-label="创建标签"
        @click="startCreate"
      >
        +
      </button>
      <input
        v-else
        ref="createInput"
        v-model="newName"
        class="tags-bar__input"
        type="text"
        placeholder="标签名"
        @keydown.enter="confirmCreate"
        @keydown.esc="cancelCreate"
        @blur="cancelCreate"
      />
    </div>
    <div v-if="nameError" class="tags-bar__name-error">
      <span class="tags-bar__error-text">{{ nameError }}</span>
    </div>
    <div v-if="error" class="tags-bar__error">
      <span class="tags-bar__error-text">{{ error }}</span>
      <button type="button" class="tags-bar__retry" @click="emit('retry')">重试</button>
    </div>
  </div>
</template>

<style scoped>
.tags-bar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.tags-bar__all {
  border: 1px solid var(--color-border);
  background-color: var(--color-surface);
  color: var(--color-text-secondary);
  border-radius: var(--radius-full);
  padding: 2px var(--space-3);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.tags-bar__all--active {
  background-color: var(--color-accent);
  color: var(--color-on-accent);
  border-color: var(--color-accent);
}

.tags-bar__list {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.tags-bar__skeleton {
  width: 48px;
  height: 20px;
  border-radius: var(--radius-full);
  background-color: var(--color-surface-hover);
}

.tags-bar__empty {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-sm);
}

.tags-bar__add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-full);
  background-color: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-md);
  line-height: 1;
  cursor: pointer;
}

.tags-bar__add:hover {
  color: var(--color-accent);
  border-color: var(--color-accent);
}

.tags-bar__input {
  width: 96px;
  box-sizing: border-box;
  background-color: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-accent);
  border-radius: var(--radius-full);
  padding: 2px var(--space-3);
  font-family: var(--font-family);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
  outline: none;
}

.tags-bar__error {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  flex-basis: 100%;
}

.tags-bar__name-error {
  flex-basis: 100%;
}

.tags-bar__name-error .tags-bar__error-text {
  color: var(--color-warning);
}

.tags-bar__error-text {
  color: var(--color-warning);
  font-size: var(--font-size-xs);
}

.tags-bar__retry {
  border: none;
  background: transparent;
  color: var(--color-accent);
  font-size: var(--font-size-xs);
  cursor: pointer;
}
</style>