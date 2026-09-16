// 备忘录相关类型定义，逐字段对齐 docs/planner-轻量强化包.md 第 2.1 节契约。

/** 备忘录实体（与 Rust `Memo` 结构体字段/顺序严格一致） */
export interface Memo {
  id: number;
  title: string;
  content: string;
  /** 创建时间，ISO8601 UTC，形如 2026-09-14T03:15:00.123Z */
  created_at: string;
  /** 更新时间，ISO8601 UTC */
  updated_at: string;
  /** 提醒时间，ISO8601 UTC，空串表示不提醒 */
  remind_at: string;
}

/** 标签实体 */
export interface Tag {
  id: number;
  name: string;
  /** 创建时间，ISO8601 UTC */
  created_at: string;
}

/** list_tags 返回：标签 + 被引用计数（标签栏「随用随现」用 memo_count>0 过滤） */
export interface TagWithCount extends Tag {
  memo_count: number;
}

// ===== 备忘录（P0 输入不变，仅返回增益 remind_at）=====
export interface CreateMemoInput {
  title: string;
  content: string;
}

export interface GetMemoInput {
  id: number;
}

export interface UpdateMemoInput {
  id: number;
  title: string;
  content: string;
}

export interface DeleteMemoInput {
  id: number;
}

export interface DeleteMemoOutput {
  id: number;
}

export interface SearchMemosInput {
  keyword: string;
}

// ===== 标签 =====
export interface CreateTagInput {
  name: string;
}

export interface DeleteTagInput {
  id: number;
}

export interface DeleteTagOutput {
  id: number;
}

export interface ListMemoTagsInput {
  memo_id: number;
}

export interface SetMemoTagsInput {
  memo_id: number;
  tag_names: string[];
}

export interface ListMemosByTagInput {
  tag_id: number;
  keyword?: string;
}

// ===== 提醒 =====
export interface SetMemoReminderInput {
  memo_id: number;
  remind_at: string;
}

export interface ClearMemoReminderInput {
  memo_id: number;
}

// ===== 应用配置 =====
export type ThemeMode = 'system' | 'light' | 'dark';
export type CloseBehavior = 'quit' | 'hide';

export interface ShortcutBindings {
  new_memo: string;
  save: string;
  delete: string;
  focus_search: string;
  quick_note: string;
}

export interface AppSettings {
  theme: ThemeMode;
  close_behavior: CloseBehavior;
  shortcuts: ShortcutBindings;
}

export interface UpdateSettingsInput {
  theme?: ThemeMode;
  close_behavior?: CloseBehavior;
  shortcuts?: ShortcutBindings;
}

// ===== 错误（沿用）=====
export type AppErrorCode = 'E_VALIDATION' | 'E_NOT_FOUND' | 'E_INTERNAL';

export interface AppError {
  code: AppErrorCode;
  message: string;
}

// ===== 视图状态 =====
export type View = 'list' | 'editor';

export type EditorMode = 'create' | 'edit';

// ===== 前端派生展示类型（对齐 docs/ui_design-轻量强化包.md 第 4.1 节）=====

/** 当前选中标签，null 表示「全部」 */
export type ActiveTagId = number | null;

/** 待办解析结果 */
export interface TodoPart {
  /** 原文行（含 - [ ] / - [x]） */
  line: string;
  /** '[x]' => true */
  checked: boolean;
  /** 行内文本（不含标记） */
  text: string;
}

/** 提醒展示信息 */
export interface ReminderDisplay {
  /** ISO8601 UTC 或 ''（空 = 无提醒） */
  remindAt: string;
  /** 前端据本地时间判断是否已过期 */
  overdue: boolean;
}