import { isTauri } from "@tauri-apps/api/core";
import { create } from "zustand";
import { api } from "../lib/tauri";
import type { ReminderRule, ReminderSnapshot } from "../types/reminder";

type ReminderState = {
  snapshot: ReminderSnapshot | null;
  error: string | null;
  refresh: () => Promise<void>;
  updateRule: (rule: ReminderRule, enabled: boolean, intervalMinutes: number, snoozeMinutes: number) => Promise<void>;
  updateQuiet: (start: string, end: string) => Promise<void>;
  updateAttention: (sound: boolean, repeat: boolean) => Promise<void>;
  complete: (ids: string[]) => Promise<boolean>;
  snooze: (ids: string[]) => Promise<boolean>;
};

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useReminderStore = create<ReminderState>((set, get) => ({
  snapshot: null,
  error: null,
  refresh: async () => {
    if (!isTauri()) return;
    try { set({ snapshot: await api.reminderSnapshot(), error: null }); }
    catch (error) { set({ error: message(error) }); }
  },
  updateRule: async (rule, enabled, intervalMinutes, snoozeMinutes) => {
    try {
      await api.updateReminderRule(rule.id, enabled, intervalMinutes, snoozeMinutes);
      await get().refresh();
    } catch (error) { set({ error: message(error) }); }
  },
  updateQuiet: async (start, end) => {
    try {
      await api.updateQuietHours(start, end);
      await get().refresh();
    } catch (error) { set({ error: message(error) }); }
  },
  updateAttention: async (sound, repeat) => {
    try {
      await api.updateAttentionSettings(sound, repeat);
      await get().refresh();
    } catch (error) { set({ error: message(error) }); }
  },
  complete: async (ids) => {
    try { await api.completeReminders(ids); await get().refresh(); return true; }
    catch (error) { set({ error: message(error) }); return false; }
  },
  snooze: async (ids) => {
    try { await api.snoozeReminders(ids); await get().refresh(); return true; }
    catch (error) { set({ error: message(error) }); return false; }
  },
}));
