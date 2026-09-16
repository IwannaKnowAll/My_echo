import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  createMemo,
  deleteMemo,
  getMemo,
  listMemos,
  searchMemos,
  toAppError,
  updateMemo,
} from '../lib/api';
import type { Memo } from '../types/memo';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const mockInvoke = vi.mocked(invoke);

const memo: Memo = {
  id: 1,
  title: '标题',
  content: '正文',
  created_at: '2026-09-14T03:15:00.000Z',
  updated_at: '2026-09-14T03:15:00.000Z',
  remind_at: '',
};

beforeEach(() => {
  mockInvoke.mockReset();
});

describe('api 六函数：命令字符串与入参按 Rust 参数名包装', () => {
  it('createMemo 以 { input } 包装 title/content 并返回 Memo', async () => {
    mockInvoke.mockResolvedValue(memo);
    const result = await createMemo({ title: '标题', content: '正文' });
    expect(mockInvoke).toHaveBeenCalledWith('create_memo', {
      input: { title: '标题', content: '正文' },
    });
    expect(result).toEqual(memo);
  });

  it('listMemos 无入参调用 list_memos', async () => {
    mockInvoke.mockResolvedValue([memo]);
    const result = await listMemos();
    expect(mockInvoke).toHaveBeenCalledWith('list_memos');
    expect(result).toEqual([memo]);
  });

  it('getMemo 以 { input } 包装 id', async () => {
    mockInvoke.mockResolvedValue(memo);
    await getMemo({ id: 1 });
    expect(mockInvoke).toHaveBeenCalledWith('get_memo', { input: { id: 1 } });
  });

  it('updateMemo 以 { input } 包装 id/title/content', async () => {
    mockInvoke.mockResolvedValue(memo);
    await updateMemo({ id: 1, title: '标题', content: '正文' });
    expect(mockInvoke).toHaveBeenCalledWith('update_memo', {
      input: { id: 1, title: '标题', content: '正文' },
    });
  });

  it('deleteMemo 以 { input } 包装 id 并返回 {id}', async () => {
    mockInvoke.mockResolvedValue({ id: 1 });
    const result = await deleteMemo({ id: 1 });
    expect(mockInvoke).toHaveBeenCalledWith('delete_memo', { input: { id: 1 } });
    expect(result).toEqual({ id: 1 });
  });

  it('searchMemos 以 { input } 包装 keyword', async () => {
    mockInvoke.mockResolvedValue([memo]);
    await searchMemos({ keyword: '关键' });
    expect(mockInvoke).toHaveBeenCalledWith('search_memos', { input: { keyword: '关键' } });
  });
});

describe('toAppError 错误规整', () => {
  it('合法 code+message 原样保留', () => {
    expect(toAppError({ code: 'E_VALIDATION', message: '标题不能为空' })).toEqual({
      code: 'E_VALIDATION',
      message: '标题不能为空',
    });
    expect(toAppError({ code: 'E_NOT_FOUND', message: '备忘录不存在' })).toEqual({
      code: 'E_NOT_FOUND',
      message: '备忘录不存在',
    });
  });

  it('结构不符时回退 E_INTERNAL 兜底文案', () => {
    expect(toAppError({ code: 'UNKNOWN', message: 'x' })).toEqual({
      code: 'E_INTERNAL',
      message: '操作失败，请重试',
    });
    expect(toAppError({ code: 'E_VALIDATION' })).toEqual({
      code: 'E_INTERNAL',
      message: '操作失败，请重试',
    });
    expect(toAppError('字符串错误')).toEqual({
      code: 'E_INTERNAL',
      message: '字符串错误',
    });
    expect(toAppError(null)).toEqual({
      code: 'E_INTERNAL',
      message: '操作失败，请重试',
    });
  });
});