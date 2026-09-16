<script setup lang="ts">
import { computed, ref } from 'vue';
import { formatSmartTime, toDatetimeLocalValue, toUtcIso } from '../lib/time';

const props = defineProps<{
  memoId: number;
  remindAt: string;
}>();

const emit = defineEmits<{
  change: [remindAt: string];
}>();

const editing = ref(false);
const localValue = ref('');
const error = ref('');

const displayTime = computed(() => formatSmartTime(props.remindAt));

function startEdit(): void {
  error.value = '';
  localValue.value = props.remindAt ? toDatetimeLocalValue(props.remindAt) : '';
  editing.value = true;
}

function cancelEdit(): void {
  editing.value = false;
  error.value = '';
}

function confirmSet(): void {
  const utc = toUtcIso(localValue.value);
  if (utc === '') {
    error.value = '提醒时间格式不正确';
    return;
  }
  const minute = 60_000;
  if (Math.floor(new Date(utc).getTime() / minute) < Math.floor(Date.now() / minute)) {
    error.value = '提醒时间不能早于当前时间';
    return;
  }
  error.value = '';
  editing.value = false;
  emit('change', utc);
}

function clearReminder(): void {
  error.value = '';
  editing.value = false;
  emit('change', '');
}
</script>

<template>
  <div class="reminder">
    <template v-if="!editing">
      <button v-if="!remindAt" type="button" class="reminder__add" @click="startEdit">
        添加提醒
      </button>
      <template v-else>
        <span class="reminder__icon" aria-hidden="true">🔔</span>
        <span class="reminder__time">{{ displayTime }}</span>
        <button type="button" class="reminder__link" @click="startEdit">修改</button>
        <button type="button" class="reminder__link" @click="clearReminder">清除</button>
      </template>
    </template>
    <template v-else>
      <input v-model="localValue" class="reminder__picker" type="datetime-local" />
      <button type="button" class="reminder__link" @click="confirmSet">设置</button>
      <button type="button" class="reminder__link" @click="cancelEdit">取消</button>
    </template>
    <p v-if="error" class="reminder__error">{{ error }}</p>
  </div>
</template>

<style scoped>
.reminder {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-2);
  min-height: 24px;
}

.reminder__icon {
  font-size: var(--font-size-sm);
  line-height: 1;
}

.reminder__time {
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
}

.reminder__add {
  border: none;
  background: transparent;
  color: var(--color-accent);
  font-size: var(--font-size-sm);
  cursor: pointer;
  padding: 0;
}

.reminder__link {
  border: none;
  background: transparent;
  color: var(--color-accent);
  font-size: var(--font-size-sm);
  cursor: pointer;
  padding: 0;
}

.reminder__picker {
  box-sizing: border-box;
  background-color: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-1) var(--space-2);
  font-family: var(--font-family);
  font-size: var(--font-size-sm);
  outline: none;
}

.reminder__picker:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
}

.reminder__error {
  flex-basis: 100%;
  margin: 0;
  color: var(--color-danger);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
}
</style>