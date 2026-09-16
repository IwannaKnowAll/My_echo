import { describe, expect, it } from 'vitest';
import { formatDateTime, formatSmartTime } from '../lib/time';

describe('formatDateTime', () => {
  it('非法输入返回空字符串', () => {
    expect(formatDateTime('')).toBe('');
    expect(formatDateTime('not-a-date')).toBe('');
  });

  it('ISO8601 UTC 转换为本地 YYYY-MM-DD HH:mm', () => {
    // 用本地时间构造，再取 UTC ISO 传入，结果应与本地墙钟一致（与运行环境时区无关）。
    const local = new Date(2026, 0, 2, 13, 5, 0);
    expect(formatDateTime(local.toISOString())).toBe('2026-01-02 13:05');
  });

  it('个位数月份/日/时分补零', () => {
    const local = new Date(2026, 0, 2, 9, 5, 0);
    expect(formatDateTime(local.toISOString())).toBe('2026-01-02 09:05');
  });
});

describe('formatSmartTime', () => {
  it('非法输入返回空字符串', () => {
    expect(formatSmartTime('nope')).toBe('');
  });

  it('非当天展示完整日期时间', () => {
    const past = new Date(2000, 0, 1, 8, 30, 0);
    expect(formatSmartTime(past.toISOString())).toBe('2000-01-01 08:30');
  });

  it('当天仅展示 HH:mm', () => {
    expect(formatSmartTime(new Date().toISOString())).toMatch(/^\d{2}:\d{2}$/);
  });
});