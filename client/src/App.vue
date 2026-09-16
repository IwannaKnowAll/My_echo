<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import AppButton from './components/AppButton.vue';
import ConfirmDialog from './components/ConfirmDialog.vue';
import EmptyState from './components/EmptyState.vue';
import ErrorBanner from './components/ErrorBanner.vue';
import MemoListItem from './components/MemoListItem.vue';
import QuickNoteView from './components/QuickNoteView.vue';
import ReminderField from './components/ReminderField.vue';
import SearchBox from './components/SearchBox.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import TagChip from './components/TagChip.vue';
import TagsBar from './components/TagsBar.vue';
import TextInput from './components/TextInput.vue';
import TodoItem from './components/TodoItem.vue';
import {
  clearMemoReminder,
  createMemo,
  createTag,
  deleteMemo,
  deleteTag,
  getMemo,
  getSettings,
  listMemoTags,
  listMemos,
  listMemosByTag,
  listTags,
  listenOpenMemo,
  listenQuickNoteSaved,
  searchMemos,
  setMemoReminder,
  setMemoTags,
  toAppError,
  updateMemo,
  updateSettings,
} from './lib/api';
import { DEFAULT_SHORTCUTS, matchesCombo } from './lib/shortcuts';
import { initTheme, setThemeMode } from './lib/theme';
import { parseTodoLines, toggleTodoLine } from './lib/todo';
import { formatSmartTime } from './lib/time';
import type {
  ActiveTagId,
  AppError,
  AppSettings,
  EditorMode,
  Memo,
  Tag,
  TagWithCount,
  UpdateSettingsInput,
  View,
} from './types/memo';

const isQuickMode =
  typeof window !== 'undefined' && window.location.search.includes('mode=quick');

const view = ref<View>('list');
const editorMode = ref<EditorMode>('create');
const editingId = ref<number | null>(null);

// 列表状态
const memos = ref<Memo[]>([]);
const listLoading = ref(false);
const listError = ref('');

// 搜索状态
const searchKeyword = ref('');
let searchTimer: ReturnType<typeof setTimeout> | null = null;
let listRequestId = 0;

// 标签栏状态
const tags = ref<TagWithCount[]>([]);
const tagsLoading = ref(false);
const tagsError = ref('');
const activeTagId = ref<ActiveTagId>(null);

// 编辑表单状态
const formTitle = ref('');
const formContent = ref('');
const formLoading = ref(false);
const saving = ref(false);
const formError = ref('');
const titleError = ref('');
const editorTags = ref<Tag[]>([]);
const tagInput = ref('');
const remindAt = ref('');

// 删除确认状态（备忘录 / 标签共用弹窗）
const confirmVisible = ref(false);
const confirmMessage = ref('');
const deleting = ref(false);
const confirmKind = ref<'memo' | 'tag'>('memo');
const deletingTag = ref<TagWithCount | null>(null);

// 列表视图顶部提示（E_NOT_FOUND 等场景返回列表后展示）
const listNotice = ref('');

// 设置面板状态
const settings = ref<AppSettings | null>(null);
const settingsOpen = ref(false);
const settingsLoading = ref(false);
const settingsSaving = ref(false);
const settingsError = ref('');

const unlisteners: Array<() => void> = [];

const NOT_FOUND_MESSAGE = '该备忘录不存在或已被删除';

function friendlyMessage(appError: AppError): string {
  if (appError.code === 'E_VALIDATION') {
    return appError.message;
  }
  if (appError.code === 'E_INTERNAL' && typeof appError.message === 'string' && appError.message !== '') {
    return appError.message;
  }
  return '操作失败，请重试';
}

async function loadList(): Promise<void> {
  const reqId = ++listRequestId;
  listLoading.value = true;
  listError.value = '';
  try {
    const keyword = searchKeyword.value.trim();
    const tagId = activeTagId.value;
    let data: Memo[];
    if (keyword === '' && tagId === null) {
      data = await listMemos();
    } else if (tagId !== null) {
      data = await listMemosByTag(
        keyword === '' ? { tag_id: tagId } : { tag_id: tagId, keyword },
      );
    } else {
      data = await searchMemos({ keyword });
    }
    if (reqId === listRequestId) {
      memos.value = data;
    }
  } catch (err) {
    console.error('IPC 错误', err);
    if (reqId === listRequestId) {
      listError.value = friendlyMessage(toAppError(err));
      memos.value = [];
    }
  } finally {
    if (reqId === listRequestId) {
      listLoading.value = false;
    }
  }
}

async function loadTags(): Promise<void> {
  tagsLoading.value = true;
  tagsError.value = '';
  try {
    tags.value = await listTags();
  } catch (err) {
    tagsError.value = friendlyMessage(toAppError(err));
  } finally {
    tagsLoading.value = false;
  }
}

const visibleTags = computed(() => tags.value.filter((tag) => tag.memo_count > 0));

const tagSuggestions = computed(() => {
  const query = tagInput.value.trim().toLowerCase();
  if (query === '') {
    return [];
  }
  return tags.value
    .filter(
      (tag) =>
        tag.name.toLowerCase().includes(query) &&
        !editorTags.value.some((selected) => selected.name.toLowerCase() === tag.name.toLowerCase()),
    )
    .slice(0, 5);
});

const todoLines = computed(() => parseTodoLines(formContent.value));

const emptyState = computed(() => {
  const keyword = searchKeyword.value.trim();
  const tagActive = activeTagId.value !== null;
  if (tagActive && keyword === '') {
    return {
      type: 'empty' as const,
      title: '该标签下暂无备忘录',
      description: '换个标签，或点右上角新建备忘录并打上该标签',
    };
  }
  if (tagActive && keyword !== '') {
    return {
      type: 'search' as const,
      title: '没有找到相关内容',
      description: '在当前标签下换个关键词试试',
    };
  }
  return { type: keyword === '' ? ('empty' as const) : ('search' as const) };
});

const confirmTitle = computed(() =>
  confirmKind.value === 'memo' ? '删除备忘录' : '删除标签',
);

function scheduleSearch(value: string): void {
  if (searchTimer !== null) {
    clearTimeout(searchTimer);
    searchTimer = null;
  }
  const keyword = value.trim();
  if (keyword === '') {
    void loadList();
    return;
  }
  searchTimer = setTimeout(() => {
    searchTimer = null;
    void loadList();
  }, 300);
}

function onSearchInput(value: string): void {
  searchKeyword.value = value;
  listNotice.value = '';
  scheduleSearch(value);
}

function onSearchClear(): void {
  searchKeyword.value = '';
  listNotice.value = '';
  if (searchTimer !== null) {
    clearTimeout(searchTimer);
    searchTimer = null;
  }
  void loadList();
}

function onTagSelect(tagId: number | null): void {
  activeTagId.value = tagId;
  void loadList();
}

async function onTagCreate(name: string): Promise<void> {
  try {
    await createTag({ name });
    await loadTags();
  } catch (err) {
    listNotice.value = friendlyMessage(toAppError(err));
  }
}

function openTagConfirm(tag: TagWithCount): void {
  deletingTag.value = tag;
  confirmKind.value = 'tag';
  confirmMessage.value = `确定删除标签「${tag.name}」吗？已关联的备忘录会解除该标签。`;
  confirmVisible.value = true;
}

function startCreate(): void {
  editingId.value = null;
  editorMode.value = 'create';
  formTitle.value = '';
  formContent.value = '';
  editorTags.value = [];
  tagInput.value = '';
  remindAt.value = '';
  formError.value = '';
  titleError.value = '';
  listNotice.value = '';
  view.value = 'editor';
}

async function openEditorById(id: number): Promise<void> {
  editingId.value = id;
  editorMode.value = 'edit';
  formTitle.value = '';
  formContent.value = '';
  editorTags.value = [];
  tagInput.value = '';
  remindAt.value = '';
  formError.value = '';
  titleError.value = '';
  listNotice.value = '';
  view.value = 'editor';
  formLoading.value = true;
  try {
    const [full, memoTags] = await Promise.all([
      getMemo({ id }),
      listMemoTags({ memo_id: id }),
    ]);
    formTitle.value = full.title;
    formContent.value = full.content;
    remindAt.value = full.remind_at;
    editorTags.value = memoTags;
  } catch (err) {
    console.error('IPC 错误', err);
    const appError = toAppError(err);
    if (appError.code === 'E_NOT_FOUND') {
      listNotice.value = NOT_FOUND_MESSAGE;
      view.value = 'list';
      await loadList();
    } else {
      formError.value = friendlyMessage(appError);
    }
  } finally {
    formLoading.value = false;
  }
}

function goBack(): void {
  view.value = 'list';
}

function addTagFromInput(): void {
  const name = tagInput.value.trim();
  tagInput.value = '';
  if (name === '') {
    return;
  }
  if (editorTags.value.some((tag) => tag.name.toLowerCase() === name.toLowerCase())) {
    return;
  }
  const existing = tags.value.find((tag) => tag.name.toLowerCase() === name.toLowerCase());
  editorTags.value.push(existing ?? { id: 0, name, created_at: '' });
}

function suggestSelect(tag: Tag): void {
  if (editorTags.value.some((selected) => selected.name.toLowerCase() === tag.name.toLowerCase())) {
    tagInput.value = '';
    return;
  }
  editorTags.value.push(tag);
  tagInput.value = '';
}

function removeEditorTag(tag: Tag): void {
  editorTags.value = editorTags.value.filter(
    (selected) => selected.name.toLowerCase() !== tag.name.toLowerCase(),
  );
}

async function saveTodoContent(): Promise<void> {
  const id = editingId.value;
  if (id === null) {
    return;
  }
  try {
    await updateMemo({ id, title: formTitle.value, content: formContent.value });
  } catch (err) {
    formError.value = friendlyMessage(toAppError(err));
  }
}

function onTodoToggle(index: number): void {
  formContent.value = toggleTodoLine(formContent.value, index);
  if (editorMode.value === 'edit' && editingId.value !== null) {
    const title = formTitle.value.trim();
    if (title === '') {
      return;
    }
    void saveTodoContent();
  }
}

async function onReminderChange(nextRemindAt: string): Promise<void> {
  const id = editingId.value;
  if (id === null) {
    return;
  }
  formError.value = '';
  try {
    if (nextRemindAt === '') {
      const updated = await clearMemoReminder({ memo_id: id });
      remindAt.value = updated.remind_at;
    } else {
      const updated = await setMemoReminder({ memo_id: id, remind_at: nextRemindAt });
      remindAt.value = updated.remind_at;
    }
  } catch (err) {
    formError.value = friendlyMessage(toAppError(err));
  }
}

async function onSave(): Promise<void> {
  if (saving.value) {
    return;
  }
  const trimmed = formTitle.value.trim();
  if (trimmed === '') {
    titleError.value = '标题不能为空';
    return;
  }
  if ([...formTitle.value].length > 100) {
    titleError.value = '标题不能超过 100 个字符';
    return;
  }
  const id = editingId.value;
  if (editorMode.value === 'edit' && id === null) {
    formError.value = '操作失败，请重试';
    return;
  }
  titleError.value = '';
  formError.value = '';
  saving.value = true;
  try {
    let memoId: number;
    if (editorMode.value === 'create') {
      const created = await createMemo({ title: formTitle.value, content: formContent.value });
      memoId = created.id;
    } else {
      memoId = id as number;
      await updateMemo({ id: memoId, title: formTitle.value, content: formContent.value });
    }
    const tagNames = editorTags.value.map((tag) => tag.name);
    if (editorMode.value === 'edit' || tagNames.length > 0) {
      await setMemoTags({ memo_id: memoId, tag_names: tagNames });
    }
    view.value = 'list';
    await loadList();
    await loadTags();
  } catch (err) {
    const appError = toAppError(err);
    if (appError.code === 'E_NOT_FOUND') {
      listNotice.value = NOT_FOUND_MESSAGE;
      view.value = 'list';
      await loadList();
    } else if (appError.code === 'E_VALIDATION') {
      titleError.value = appError.message;
    } else {
      formError.value = friendlyMessage(appError);
    }
  } finally {
    saving.value = false;
  }
}

function openMemoConfirm(): void {
  if (editingId.value === null) {
    return;
  }
  confirmKind.value = 'memo';
  confirmMessage.value = `确定要删除“${formTitle.value}”吗？此操作不可撤销。`;
  confirmVisible.value = true;
}

function cancelDelete(): void {
  confirmVisible.value = false;
}

async function deleteMemoConfirmed(): Promise<void> {
  const id = editingId.value;
  if (id === null) {
    return;
  }
  deleting.value = true;
  try {
    await deleteMemo({ id });
    confirmVisible.value = false;
    view.value = 'list';
    await loadList();
  } catch (err) {
    const appError = toAppError(err);
    confirmVisible.value = false;
    if (appError.code === 'E_NOT_FOUND') {
      listNotice.value = NOT_FOUND_MESSAGE;
      view.value = 'list';
      await loadList();
    } else {
      formError.value = friendlyMessage(appError);
    }
  } finally {
    deleting.value = false;
  }
}

async function deleteTagConfirmed(): Promise<void> {
  const tag = deletingTag.value;
  if (tag === null) {
    return;
  }
  deleting.value = true;
  try {
    await deleteTag({ id: tag.id });
    confirmVisible.value = false;
    if (activeTagId.value === tag.id) {
      activeTagId.value = null;
    }
    await loadTags();
    await loadList();
  } catch (err) {
    confirmVisible.value = false;
    listNotice.value = friendlyMessage(toAppError(err));
  } finally {
    deleting.value = false;
  }
}

function confirmDelete(): void {
  if (deleting.value) {
    return;
  }
  if (confirmKind.value === 'tag') {
    void deleteTagConfirmed();
    return;
  }
  void deleteMemoConfirmed();
}

// ===== 设置面板 =====

async function openSettings(): Promise<void> {
  settingsOpen.value = true;
  settingsLoading.value = true;
  settingsError.value = '';
  try {
    settings.value = await getSettings();
  } catch (err) {
    console.error('IPC 错误', err);
    settingsError.value = friendlyMessage(toAppError(err));
  } finally {
    settingsLoading.value = false;
  }
}

function closeSettings(): void {
  settingsOpen.value = false;
  settingsError.value = '';
}

async function onSettingsUpdate(patch: UpdateSettingsInput): Promise<void> {
  if (settingsSaving.value) {
    return;
  }
  settingsSaving.value = true;
  settingsError.value = '';
  try {
    const updated = await updateSettings(patch);
    settings.value = updated;
    setThemeMode(updated.theme);
  } catch (err) {
    settingsError.value = friendlyMessage(toAppError(err));
  } finally {
    settingsSaving.value = false;
  }
}

// ===== 应用内快捷键 =====

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const tagName = target.tagName.toLowerCase();
  return tagName === 'input' || tagName === 'textarea' || target.isContentEditable;
}

async function focusSearch(): Promise<void> {
  view.value = 'list';
  await nextTick();
  document.querySelector<HTMLInputElement>('.search__input')?.focus();
}

function onGlobalKeydown(event: KeyboardEvent): void {
  const shortcuts = settings.value?.shortcuts ?? DEFAULT_SHORTCUTS;

  if (matchesCombo(event, shortcuts.save)) {
    if (view.value === 'editor') {
      event.preventDefault();
      void onSave();
    }
    return;
  }
  if (isEditableTarget(event.target)) {
    return;
  }
  if (matchesCombo(event, shortcuts.new_memo)) {
    if (view.value === 'list') {
      event.preventDefault();
      startCreate();
    }
    return;
  }
  if (matchesCombo(event, shortcuts.focus_search)) {
    event.preventDefault();
    void focusSearch();
    return;
  }
  if (matchesCombo(event, shortcuts.delete)) {
    if (view.value === 'editor' && editorMode.value === 'edit') {
      event.preventDefault();
      openMemoConfirm();
    }
  }
}

// ===== 初始化 =====

async function initSettings(): Promise<void> {
  settings.value = await initTheme();
}

async function setupEventListeners(): Promise<void> {
  try {
    unlisteners.push(
      await listenOpenMemo((payload) => {
        void openEditorById(payload.memo_id);
      }),
    );
  } catch {
    // 事件监听失败不阻塞
  }
  try {
    unlisteners.push(
      await listenQuickNoteSaved(() => {
        void loadList();
        void loadTags();
      }),
    );
  } catch {
    // 事件监听失败不阻塞
  }
}

onMounted(() => {
  void initSettings();
  void loadList();
  void loadTags();
  void setupEventListeners();
  window.addEventListener('keydown', onGlobalKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onGlobalKeydown);
  if (searchTimer !== null) {
    clearTimeout(searchTimer);
  }
  for (const unlisten of unlisteners) {
    unlisten();
  }
});
</script>

<template>
  <QuickNoteView v-if="isQuickMode" />
  <div v-else class="app">
    <!-- 列表视图 -->
    <template v-if="view === 'list'">
      <header class="toolbar">
        <h1 class="app-title">My Echo</h1>
        <div class="toolbar__actions">
          <AppButton label="设置" variant="secondary" @click="openSettings" />
          <AppButton label="新建备忘录" variant="primary" @click="startCreate" />
        </div>
      </header>
      <div class="tags-area">
        <TagsBar
          :tags="visibleTags"
          :active-tag-id="activeTagId"
          :loading="tagsLoading"
          :error="tagsError"
          @select="onTagSelect"
          @create="onTagCreate"
          @remove="openTagConfirm"
          @retry="loadTags"
        />
      </div>
      <div class="search-area">
        <SearchBox
          :model-value="searchKeyword"
          placeholder="搜索标题或内容"
          @update:model-value="onSearchInput"
          @clear="onSearchClear"
        />
      </div>
      <div v-if="listNotice" class="list-notice">
        <ErrorBanner :message="listNotice" @close="listNotice = ''" />
      </div>
      <main class="list-area">
        <div v-if="listLoading" class="center-state">
          <p class="center-state__text">加载中…</p>
        </div>
        <div v-else-if="listError" class="center-state">
          <p class="center-state__text">{{ listError }}</p>
          <AppButton label="重试" @click="loadList" />
        </div>
        <EmptyState
          v-else-if="memos.length === 0"
          :type="emptyState.type"
          :title="emptyState.title"
          :description="emptyState.description"
          @action="startCreate"
        />
        <ul v-else class="memo-list">
          <li v-for="memo in memos" :key="memo.id">
            <MemoListItem
              :memo="memo"
              :time-text="formatSmartTime(memo.updated_at)"
              @click="openEditorById(memo.id)"
            />
          </li>
        </ul>
      </main>
    </template>

    <!-- 编辑视图 -->
    <template v-else>
      <header class="toolbar">
        <AppButton label="返回" variant="ghost" @click="goBack" />
        <AppButton
          v-if="editorMode === 'edit'"
          label="删除"
          variant="danger"
          @click="openMemoConfirm"
        />
      </header>
      <main class="editor-area">
        <ErrorBanner
          v-if="formError"
          :message="formError"
          @close="formError = ''"
        />
        <div v-if="formLoading" class="center-state editor-loading">
          <p class="center-state__text">加载中…</p>
        </div>
        <form v-else class="editor-form" @submit.prevent="onSave">
          <div class="editor-title">
            <TextInput
              v-model="formTitle"
              placeholder="请输入备忘录标题"
              :error="titleError"
              :disabled="saving"
              :autofocus="editorMode === 'create'"
            />
            <span
              class="title-counter"
              :class="{ 'title-counter--over': [...formTitle].length > 100 }"
            >
              {{ [...formTitle].length }} / 100
            </span>
          </div>

          <div class="editor-tags">
            <TagChip
              v-for="tag in editorTags"
              :key="`${tag.name}-${tag.id}`"
              :tag="tag"
              removable
              :disabled="saving"
              @remove="removeEditorTag(tag)"
            />
            <div class="editor-tag-input">
              <input
                v-model="tagInput"
                class="editor-tag-field"
                type="text"
                placeholder="添加标签"
                :disabled="saving"
                @keydown.enter.prevent="addTagFromInput"
              />
              <ul v-if="tagSuggestions.length" class="tag-suggest">
                <li
                  v-for="suggestion in tagSuggestions"
                  :key="suggestion.id"
                  class="tag-suggest__item"
                  @mousedown.prevent="suggestSelect(suggestion)"
                >
                  {{ suggestion.name }}
                </li>
              </ul>
            </div>
          </div>

          <ReminderField
            v-if="editorMode === 'edit' && editingId !== null"
            :memo-id="editingId"
            :remind-at="remindAt"
            @change="onReminderChange"
          />

          <TextInput
            v-model="formContent"
            class="editor-content"
            placeholder="正文（可选）"
            :disabled="saving"
            multiline
          />

          <div v-if="todoLines.length" class="editor-todos">
            <TodoItem
              v-for="(line, index) in todoLines"
              :key="index"
              :checked="line.checked"
              :text="line.text"
              :disabled="saving"
              @toggle="onTodoToggle(index)"
            />
          </div>

          <div class="editor-actions">
            <AppButton label="取消" variant="secondary" @click="goBack" />
            <AppButton
              label="保存"
              variant="primary"
              type="submit"
              :loading="saving"
            />
          </div>
        </form>
      </main>
    </template>

    <!-- 删除确认弹窗 -->
    <ConfirmDialog
      :visible="confirmVisible"
      :title="confirmTitle"
      :message="confirmMessage"
      :loading="deleting"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />

    <!-- 设置面板 -->
    <SettingsPanel
      :visible="settingsOpen"
      :settings="settings"
      :error="settingsError"
      @update="onSettingsUpdate"
      @close="closeSettings"
    />
  </div>
</template>

<style scoped>
.app {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  flex-shrink: 0;
}

.toolbar__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.app-title {
  margin: 0;
  font-size: var(--font-size-xl);
  font-weight: var(--font-weight-semibold);
  line-height: var(--line-height-tight);
  color: var(--color-text);
}

.tags-area {
  padding: 0 var(--space-4) var(--space-2);
  flex-shrink: 0;
}

.search-area {
  padding: 0 var(--space-4) var(--space-3);
  flex-shrink: 0;
}

.list-notice {
  padding: 0 var(--space-4) var(--space-3);
  flex-shrink: 0;
}

.list-area {
  flex: 1;
  overflow-y: auto;
  padding: 0 var(--space-4) var(--space-4);
}

.memo-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.center-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-4);
  padding: var(--space-8);
  text-align: center;
}

.center-state__text {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
}

.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 0 var(--space-4) var(--space-4);
  overflow-y: auto;
  gap: var(--space-4);
}

.editor-loading {
  flex: 1;
}

.editor-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.editor-title {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.title-counter {
  align-self: flex-end;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
  line-height: var(--line-height-tight);
}

.title-counter--over {
  color: var(--color-danger);
}

.editor-tags {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-2);
}

.editor-tag-input {
  position: relative;
}

.editor-tag-field {
  box-sizing: border-box;
  min-width: 96px;
  background-color: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-full);
  padding: 2px var(--space-3);
  font-family: var(--font-family);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-tight);
  outline: none;
}

.editor-tag-field:focus {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-focus);
}

.tag-suggest {
  position: absolute;
  top: calc(100% + var(--space-1));
  left: 0;
  z-index: 10;
  margin: 0;
  padding: var(--space-1);
  list-style: none;
  min-width: 120px;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-md);
}

.tag-suggest__item {
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  font-size: var(--font-size-sm);
  cursor: pointer;
}

.tag-suggest__item:hover {
  background-color: var(--color-surface-hover);
}

.editor-todos {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2) var(--space-3);
  background-color: var(--color-surface);
}

.editor-content {
  flex: 1;
}

.editor-content :deep(.control--textarea) {
  flex: 1;
  min-height: 240px;
  font-size: var(--font-size-sm);
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  flex-shrink: 0;
}
</style>