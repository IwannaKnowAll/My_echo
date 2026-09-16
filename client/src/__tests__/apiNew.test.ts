import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';
import {
  clearMemoReminder,
  createTag,
  deleteTag,
  emitQuickNoteSaved,
  getSettings,
  listenOpenMemo,
  listenQuickNoteSaved,
  listMemoTags,
  listMemosByTag,
  listTags,
  openNotificationSettings,
  setMemoReminder,
  setMemoTags,
  updateSettings,
} from '../lib/api';
import type { AppSettings } from '../types/memo';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
  emit: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);
const mockEmit = vi.mocked(emit);

const settings: AppSettings = {
  theme: 'system',
  close_behavior: 'quit',
  shortcuts: {
    new_memo: 'Cmd+N',
    save: 'Cmd+S',
    delete: 'Cmd+Backspace',
    focus_search: 'Cmd+F',
    quick_note: 'Cmd+Shift+Space',
  },
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockListen.mockReset();
  mockEmit.mockReset();
});

describe('标签四条命令包装', () => {
  it('listTags 无入参调用 list_tags', async () => {
    mockInvoke.mockResolvedValue([]);
    await listTags();
    expect(mockInvoke).toHaveBeenCalledWith('list_tags');
  });

  it('createTag 以 { input } 包装 name', async () => {
    mockInvoke.mockResolvedValue({ id: 1, name: '工作', created_at: 'x' });
    await createTag({ name: '工作' });
    expect(mockInvoke).toHaveBeenCalledWith('create_tag', { input: { name: '工作' } });
  });

  it('deleteTag 以 { input } 包装 id', async () => {
    mockInvoke.mockResolvedValue({ id: 1 });
    await deleteTag({ id: 1 });
    expect(mockInvoke).toHaveBeenCalledWith('delete_tag', { input: { id: 1 } });
  });

  it('listMemoTags 以 { input } 包装 memo_id', async () => {
    mockInvoke.mockResolvedValue([]);
    await listMemoTags({ memo_id: 3 });
    expect(mockInvoke).toHaveBeenCalledWith('list_memo_tags', { input: { memo_id: 3 } });
  });

  it('setMemoTags 以 { input } 包装 memo_id/tag_names', async () => {
    mockInvoke.mockResolvedValue([]);
    await setMemoTags({ memo_id: 3, tag_names: ['工作', '生活'] });
    expect(mockInvoke).toHaveBeenCalledWith('set_memo_tags', {
      input: { memo_id: 3, tag_names: ['工作', '生活'] },
    });
  });

  it('listMemosByTag 以 { input } 包装 tag_id（keyword 可选）', async () => {
    mockInvoke.mockResolvedValue([]);
    await listMemosByTag({ tag_id: 2 });
    expect(mockInvoke).toHaveBeenCalledWith('list_memos_by_tag', { input: { tag_id: 2 } });

    await listMemosByTag({ tag_id: 2, keyword: '关键词' });
    expect(mockInvoke).toHaveBeenLastCalledWith('list_memos_by_tag', {
      input: { tag_id: 2, keyword: '关键词' },
    });
  });
});

describe('提醒两条命令包装', () => {
  it('setMemoReminder 以 { input } 包装 memo_id/remind_at', async () => {
    mockInvoke.mockResolvedValue({});
    await setMemoReminder({ memo_id: 1, remind_at: '2026-09-16T02:00:00.000Z' });
    expect(mockInvoke).toHaveBeenCalledWith('set_memo_reminder', {
      input: { memo_id: 1, remind_at: '2026-09-16T02:00:00.000Z' },
    });
  });

  it('clearMemoReminder 以 { input } 包装 memo_id', async () => {
    mockInvoke.mockResolvedValue({});
    await clearMemoReminder({ memo_id: 1 });
    expect(mockInvoke).toHaveBeenCalledWith('clear_memo_reminder', { input: { memo_id: 1 } });
  });
});

describe('应用配置三条命令包装', () => {
  it('getSettings 无入参调用 get_settings', async () => {
    mockInvoke.mockResolvedValue(settings);
    await getSettings();
    expect(mockInvoke).toHaveBeenCalledWith('get_settings');
  });

  it('updateSettings 以 { input } 包装部分字段', async () => {
    mockInvoke.mockResolvedValue(settings);
    await updateSettings({ theme: 'dark' });
    expect(mockInvoke).toHaveBeenCalledWith('update_settings', { input: { theme: 'dark' } });
  });

  it('openNotificationSettings 无参且不传第二参数（C11）', async () => {
    mockInvoke.mockResolvedValue(null);
    const result = await openNotificationSettings();
    expect(mockInvoke).toHaveBeenCalledWith('open_notification_settings');
    expect(mockInvoke.mock.calls[0]).toHaveLength(1);
    expect(result).toBeNull();
  });
});

describe('事件封装', () => {
  it('listenOpenMemo 监听 open_memo 并解包 payload', async () => {
    const handler = vi.fn();
    mockListen.mockResolvedValue(() => {});
    await listenOpenMemo(handler);
    expect(mockListen).toHaveBeenCalledWith('open_memo', expect.any(Function));

    const listener = mockListen.mock.calls[0][1] as (event: {
      payload: { memo_id: number };
    }) => void;
    listener({ payload: { memo_id: 9 } });
    expect(handler).toHaveBeenCalledWith({ memo_id: 9 });
  });

  it('listenQuickNoteSaved 监听 quick_note_saved', async () => {
    const handler = vi.fn();
    mockListen.mockResolvedValue(() => {});
    await listenQuickNoteSaved(handler);
    expect(mockListen).toHaveBeenCalledWith('quick_note_saved', expect.any(Function));
  });

  it('emitQuickNoteSaved 发出 quick_note_saved（无载荷）', async () => {
    mockEmit.mockResolvedValue(undefined);
    await emitQuickNoteSaved();
    expect(mockEmit).toHaveBeenCalledWith('quick_note_saved');
  });
});