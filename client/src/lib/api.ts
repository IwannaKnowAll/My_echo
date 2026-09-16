// 数据访问层：仅通过 Tauri invoke 调用契约命令，统一把错误规整为 AppError。
import { invoke } from '@tauri-apps/api/core';
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppError,
  AppSettings,
  ClearMemoReminderInput,
  CreateMemoInput,
  CreateTagInput,
  DeleteMemoInput,
  DeleteMemoOutput,
  DeleteTagInput,
  DeleteTagOutput,
  GetMemoInput,
  ListMemoTagsInput,
  ListMemosByTagInput,
  Memo,
  SearchMemosInput,
  SetMemoReminderInput,
  SetMemoTagsInput,
  Tag,
  TagWithCount,
  UpdateMemoInput,
  UpdateSettingsInput,
} from '../types/memo';

const COMMANDS = {
  create: 'create_memo',
  list: 'list_memos',
  get: 'get_memo',
  update: 'update_memo',
  delete: 'delete_memo',
  search: 'search_memos',
  listTags: 'list_tags',
  createTag: 'create_tag',
  deleteTag: 'delete_tag',
  listMemoTags: 'list_memo_tags',
  setMemoTags: 'set_memo_tags',
  listMemosByTag: 'list_memos_by_tag',
  setMemoReminder: 'set_memo_reminder',
  clearMemoReminder: 'clear_memo_reminder',
  getSettings: 'get_settings',
  updateSettings: 'update_settings',
  openNotificationSettings: 'open_notification_settings',
} as const;

/** 判断值是否为契约约定的错误码 */
function isAppErrorCode(value: unknown): value is AppError['code'] {
  return value === 'E_VALIDATION' || value === 'E_NOT_FOUND' || value === 'E_INTERNAL';
}

/**
 * 将 invoke 抛出的错误规整为 AppError。
 * 契约约定 Rust 错误会被序列化为对象，直接含 code/message；
 * 若 err 为字符串（如 Tauri 对未知命令返回的 'Command xxx not found'），
 * 则放入 message 并归为 E_INTERNAL；其余结构不符统一回退兜底文案。
 */
export function toAppError(err: unknown): AppError {
  if (typeof err === 'string') {
    return { code: 'E_INTERNAL', message: err };
  }
  if (typeof err === 'object' && err !== null) {
    const candidate = err as { code?: unknown; message?: unknown };
    if (isAppErrorCode(candidate.code) && typeof candidate.message === 'string') {
      return { code: candidate.code, message: candidate.message };
    }
  }
  return { code: 'E_INTERNAL', message: '操作失败，请重试' };
}

// ===== 备忘录（P0） =====

/** 新建备忘录 */
export async function createMemo(input: CreateMemoInput): Promise<Memo> {
  return invoke<Memo>(COMMANDS.create, { input });
}

/** 取全部备忘录（契约保证按 updated_at 倒序） */
export async function listMemos(): Promise<Memo[]> {
  return invoke<Memo[]>(COMMANDS.list);
}

/** 按 id 单条查询 */
export async function getMemo(input: GetMemoInput): Promise<Memo> {
  return invoke<Memo>(COMMANDS.get, { input });
}

/** 更新备忘录 */
export async function updateMemo(input: UpdateMemoInput): Promise<Memo> {
  return invoke<Memo>(COMMANDS.update, { input });
}

/** 删除备忘录（硬删） */
export async function deleteMemo(input: DeleteMemoInput): Promise<DeleteMemoOutput> {
  return invoke<DeleteMemoOutput>(COMMANDS.delete, { input });
}

/** 搜索备忘录（标题 + 正文模糊匹配，结果按 updated_at 倒序） */
export async function searchMemos(input: SearchMemosInput): Promise<Memo[]> {
  return invoke<Memo[]>(COMMANDS.search, { input });
}

// ===== 标签 =====

/** 列出全部标签（含被引用计数） */
export async function listTags(): Promise<TagWithCount[]> {
  return invoke<TagWithCount[]>(COMMANDS.listTags);
}

/** 创建标签（管理入口） */
export async function createTag(input: CreateTagInput): Promise<Tag> {
  return invoke<Tag>(COMMANDS.createTag, { input });
}

/** 删除标签（仅解除关联，不删备忘录） */
export async function deleteTag(input: DeleteTagInput): Promise<DeleteTagOutput> {
  return invoke<DeleteTagOutput>(COMMANDS.deleteTag, { input });
}

/** 查询某备忘录的标签（编辑器回填） */
export async function listMemoTags(input: ListMemoTagsInput): Promise<Tag[]> {
  return invoke<Tag[]>(COMMANDS.listMemoTags, { input });
}

/** 整体设置某备忘录标签集合（覆盖式） */
export async function setMemoTags(input: SetMemoTagsInput): Promise<Tag[]> {
  return invoke<Tag[]>(COMMANDS.setMemoTags, { input });
}

/** 按标签筛选（可与关键词叠加） */
export async function listMemosByTag(input: ListMemosByTagInput): Promise<Memo[]> {
  return invoke<Memo[]>(COMMANDS.listMemosByTag, { input });
}

// ===== 提醒 =====

/** 设置提醒时间 */
export async function setMemoReminder(input: SetMemoReminderInput): Promise<Memo> {
  return invoke<Memo>(COMMANDS.setMemoReminder, { input });
}

/** 清除提醒 */
export async function clearMemoReminder(input: ClearMemoReminderInput): Promise<Memo> {
  return invoke<Memo>(COMMANDS.clearMemoReminder, { input });
}

// ===== 应用配置 =====

/** 读取配置（首读无记录会返回默认值并回写） */
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>(COMMANDS.getSettings);
}

/** 更新配置（部分字段） */
export async function updateSettings(input: UpdateSettingsInput): Promise<AppSettings> {
  return invoke<AppSettings>(COMMANDS.updateSettings, { input });
}

/** 跳转系统通知设置（无参数，返回 null） */
export async function openNotificationSettings(): Promise<null> {
  return invoke<null>(COMMANDS.openNotificationSettings);
}

// ===== 事件 =====

/** open_memo 事件载荷 */
export interface OpenMemoPayload {
  memo_id: number;
}

/** 监听「点通知打开备忘录」事件 */
export function listenOpenMemo(handler: (payload: OpenMemoPayload) => void): Promise<UnlistenFn> {
  return listen<OpenMemoPayload>('open_memo', (event) => {
    handler(event.payload);
  });
}

/** 监听「速记已保存」事件 */
export function listenQuickNoteSaved(handler: () => void): Promise<UnlistenFn> {
  return listen('quick_note_saved', () => {
    handler();
  });
}

/** 发出「速记已保存」事件（速记窗保存成功后调用） */
export async function emitQuickNoteSaved(): Promise<void> {
  await emit('quick_note_saved');
}