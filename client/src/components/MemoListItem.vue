<script setup lang="ts">
import { computed } from 'vue';
import { countTodo } from '../lib/todo';
import { formatDateTime } from '../lib/time';
import type { Memo } from '../types/memo';

const props = withDefaults(
  defineProps<{
    memo: Memo;
    selected?: boolean;
    timeText: string;
  }>(),
  {
    selected: false,
  },
);

defineEmits<{
  click: [];
}>();

const progress = computed(() => countTodo(props.memo.content));
const hasReminder = computed(() => props.memo.remind_at !== '');
const overdue = computed(() => {
  if (!hasReminder.value) {
    return false;
  }
  const time = new Date(props.memo.remind_at).getTime();
  return !Number.isNaN(time) && time < Date.now();
});
const reminderText = computed(() => formatDateTime(props.memo.remind_at));
const allDone = computed(() => progress.value.total > 0 && progress.value.done === progress.value.total);
</script>

<template>
  <button
    type="button"
    class="item"
    :class="{ 'item--selected': selected }"
    @click="$emit('click')"
  >
    <div class="item__head">
      <span class="item__title">{{ memo.title }}</span>
      <span
        v-if="hasReminder"
        class="item__bell"
        :class="{ 'item__bell--overdue': overdue }"
        :title="reminderText"
        aria-hidden="true"
      >🔔</span>
      <span class="item__time">{{ timeText }}</span>
    </div>
    <p v-if="memo.content" class="item__summary">{{ memo.content }}</p>
    <div v-if="progress.total > 0" class="item__meta">
      <span class="item__progress" :class="{ 'item__progress--done': allDone }">
        {{ progress.done }}/{{ progress.total }}
      </span>
    </div>
  </button>
</template>

<style scoped>
.item {
  width: 100%;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: var(--space-3);
  background-color: var(--color-surface);
  border: none;
  border-left: 3px solid transparent;
  border-radius: var(--radius-md);
  text-align: left;
  cursor: pointer;
  transition: background-color 0.15s ease, border-color 0.15s ease;
}

.item:hover {
  background-color: var(--color-surface-hover);
}

.item--selected {
  border-left-color: var(--color-accent);
  background-color: var(--color-surface-hover);
}

.item__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-3);
}

.item__title {
  flex: 1;
  min-width: 0;
  color: var(--color-text);
  font-size: var(--font-size-md);
  font-weight: var(--font-weight-medium);
  line-height: var(--line-height-tight);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item__bell {
  flex-shrink: 0;
  color: var(--color-reminder);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
}

.item__bell--overdue {
  color: var(--color-reminder-muted);
}

.item__time {
  flex-shrink: 0;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
}

.item__summary {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-normal);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item__meta {
  display: flex;
  justify-content: flex-end;
}

.item__progress {
  background-color: var(--color-progress-track);
  color: var(--color-text-secondary);
  border-radius: var(--radius-full);
  padding: 0 var(--space-2);
  font-size: var(--font-size-xs);
  line-height: var(--line-height-tight);
}

.item__progress--done {
  color: var(--color-success);
}
</style>