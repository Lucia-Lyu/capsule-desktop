import { invoke } from "@tauri-apps/api/core";
import type { Task } from "../types/task";
import type { ReminderSnapshot } from "../types/reminder";
import type { HistoryDay } from "../types/history";
import type { CapsuleTheme } from "../types/theme";

export const api = {
  todayTasks: () => invoke<Task[]>("get_today_tasks"),
  capsuleTheme: () => invoke<CapsuleTheme>("get_capsule_theme"),
  setCapsuleTheme: (theme: CapsuleTheme) => invoke<void>("set_capsule_theme", { theme }),
  carryCandidates: () => invoke<Task[]>("get_carry_candidates"),
  carryTasks: (ids: string[]) => invoke<void>("carry_tasks", { ids }),
  deferTaskToTomorrow: (id: string) => invoke<void>("defer_task_to_tomorrow", { id }),
  createTask: (title: string) => invoke<Task>("create_task", { title }),
  setTaskCompleted: (id: string, completed: boolean) =>
    invoke<void>("set_task_completed", { id, completed }),
  deleteTask: (id: string) => invoke<void>("delete_task", { id }),
  moveTask: (id: string, direction: -1 | 1) =>
    invoke<void>("move_task", { id, direction }),
  reminderSnapshot: () => invoke<ReminderSnapshot>("get_reminder_snapshot"),
  updateReminderRule: (id: string, enabled: boolean, intervalMinutes: number, snoozeMinutes: number) =>
    invoke<void>("update_reminder_rule", { id, enabled, intervalMinutes, snoozeMinutes }),
  updateQuietHours: (start: string, end: string) =>
    invoke<void>("update_quiet_hours", { start, end }),
  updateAttentionSettings: (sound: boolean, repeat: boolean) =>
    invoke<void>("update_attention_settings", { sound, repeat }),
  completeReminder: (id: string) => invoke<void>("complete_reminder", { id }),
  snoozeReminder: (id: string) => invoke<void>("snooze_reminder", { id }),
  history: (days: 1 | 7 | 30) => invoke<HistoryDay[]>("get_history", { days }),
};
