// 时间格式化工具：ISO8601 UTC → 本地可读文本。

/** 个位数补零 */
function pad(value: number): string {
  return value < 10 ? `0${value}` : String(value);
}

/**
 * 将 ISO8601 UTC 时间字符串格式化为本地时区文本 `YYYY-MM-DD HH:mm`。
 */
export function formatDateTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return '';
  }
  const y = date.getFullYear();
  const m = pad(date.getMonth() + 1);
  const d = pad(date.getDate());
  const hh = pad(date.getHours());
  const mm = pad(date.getMinutes());
  return `${y}-${m}-${d} ${hh}:${mm}`;
}

/**
 * 智能时间展示：当天显示 `HH:mm`，否则显示 `YYYY-MM-DD HH:mm`。
 */
export function formatSmartTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return '';
  }
  const now = new Date();
  const sameDay =
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();
  if (sameDay) {
    return `${pad(date.getHours())}:${pad(date.getMinutes())}`;
  }
  return formatDateTime(iso);
}

/**
 * 将 `<input type="datetime-local">` 的本地值（`YYYY-MM-DDTHH:mm`）转为 UTC ISO8601 毫秒、Z 结尾。
 */
export function toUtcIso(localValue: string): string {
  const date = new Date(localValue);
  if (Number.isNaN(date.getTime())) {
    return '';
  }
  return date.toISOString();
}

/**
 * 将 UTC ISO8601 转为 `<input type="datetime-local">` 的本地值（`YYYY-MM-DDTHH:mm`）。
 */
export function toDatetimeLocalValue(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return '';
  }
  const y = date.getFullYear();
  const m = pad(date.getMonth() + 1);
  const d = pad(date.getDate());
  const hh = pad(date.getHours());
  const mm = pad(date.getMinutes());
  return `${y}-${m}-${d}T${hh}:${mm}`;
}