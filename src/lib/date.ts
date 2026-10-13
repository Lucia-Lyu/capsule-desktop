import type { Task } from "../types/task";

const shortDate = new Intl.DateTimeFormat(undefined, { month: "numeric", day: "numeric" });
const fullDate = new Intl.DateTimeFormat(undefined, { year: "numeric", month: "numeric", day: "numeric" });

export function taskCreatedAt(task: Task): string {
  return task.originCreatedAt ?? task.createdAt;
}

export function formatShortDate(value: string): string {
  return shortDate.format(new Date(value));
}

export function formatFullDate(value: string): string {
  return fullDate.format(new Date(value));
}

export function formatLocalDay(value: string): string {
  return shortDate.format(new Date(`${value}T12:00:00`));
}
