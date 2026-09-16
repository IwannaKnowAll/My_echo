<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    type: 'empty' | 'search';
    title?: string;
    description?: string;
    actionLabel?: string;
  }>(),
  {
    title: undefined,
    description: undefined,
    actionLabel: '新建备忘录',
  },
);

const emit = defineEmits<{
  action: [];
}>();

const defaultContent = computed(() => {
  if (props.type === 'search') {
    return {
      title: '没有找到相关内容',
      description: '换个关键词试试',
    };
  }
  return {
    title: '还没有备忘录',
    description: '点击右上角“新建备忘录”开始记录',
  };
});

const shownTitle = computed(() => props.title ?? defaultContent.value.title);
const shownDescription = computed(() => props.description ?? defaultContent.value.description);
</script>

<template>
  <div class="empty">
    <div class="empty__icon" aria-hidden="true">📝</div>
    <p class="empty__title">{{ shownTitle }}</p>
    <p class="empty__desc">{{ shownDescription }}</p>
    <button
      v-if="type === 'empty'"
      type="button"
      class="empty__action"
      @click="emit('action')"
    >
      {{ actionLabel }}
    </button>
  </div>
</template>

<style scoped>
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-8) var(--space-4);
  text-align: center;
}

.empty__icon {
  font-size: 40px;
  line-height: 1;
  margin-bottom: var(--space-4);
}

.empty__title {
  margin: 0 0 var(--space-2);
  color: var(--color-text);
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  line-height: var(--line-height-tight);
}

.empty__desc {
  margin: 0 0 var(--space-5);
  color: var(--color-text-tertiary);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-normal);
}

.empty__action {
  background-color: var(--color-accent);
  color: #ffffff;
  border: none;
  border-radius: var(--radius-md);
  padding: var(--space-2) var(--space-4);
  font-family: var(--font-family);
  font-size: var(--font-size-md);
  font-weight: var(--font-weight-medium);
  line-height: var(--line-height-tight);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.empty__action:hover {
  background-color: var(--color-accent-hover);
}
</style>