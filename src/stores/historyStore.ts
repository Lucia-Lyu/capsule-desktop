import { isTauri } from "@tauri-apps/api/core";
import { create } from "zustand";
import { api } from "../lib/tauri";
import type { HistoryDay } from "../types/history";

type Range = 1 | 7 | 30;
type HistoryState = {
  range: Range;
  days: HistoryDay[];
  error: string | null;
  setRange: (range: Range) => Promise<void>;
  refresh: () => Promise<void>;
};

export const useHistoryStore = create<HistoryState>((set, get) => ({
  range: 7,
  days: [],
  error: null,
  setRange: async (range) => { set({ range }); await get().refresh(); },
  refresh: async () => {
    if (!isTauri()) return;
    try { set({ days: await api.history(get().range), error: null }); }
    catch (error) { set({ error: error instanceof Error ? error.message : String(error) }); }
  },
}));
