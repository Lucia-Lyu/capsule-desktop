import { create } from "zustand";
import { isTauri } from "@tauri-apps/api/core";
import { api } from "../lib/tauri";
import { isCapsuleTheme } from "../types/theme";
import type { CapsuleTheme } from "../types/theme";

export type CapsuleMode = "collapsed" | "expanded" | "reminder" | "history" | "settings";

type AppState = {
  capsuleMode: CapsuleMode;
  attentionVisible: boolean;
  theme: CapsuleTheme;
  themeError: string | null;
  themeSaving: boolean;
  setCapsuleMode: (mode: CapsuleMode) => void;
  setAttentionVisible: (visible: boolean) => void;
  loadTheme: () => Promise<void>;
  setTheme: (theme: CapsuleTheme) => Promise<void>;
};

export const useAppStore = create<AppState>((set, get) => ({
  capsuleMode: "collapsed",
  attentionVisible: false,
  theme: "sage",
  themeError: null,
  themeSaving: false,
  setCapsuleMode: (capsuleMode) => set({ capsuleMode }),
  setAttentionVisible: (attentionVisible) => set({ attentionVisible }),
  loadTheme: async () => {
    if (!isTauri()) return;
    try {
      const theme = await api.capsuleTheme();
      if (isCapsuleTheme(theme) && !get().themeSaving) set({ theme, themeError: null });
    } catch (error) {
      set({ themeError: error instanceof Error ? error.message : String(error) });
    }
  },
  setTheme: async (theme) => {
    if (get().themeSaving || get().theme === theme) return;
    const previous = get().theme;
    set({ theme, themeError: null, themeSaving: true });
    if (!isTauri()) { set({ themeSaving: false }); return; }
    try {
      await api.setCapsuleTheme(theme);
    } catch (error) {
      set({ theme: previous, themeError: error instanceof Error ? error.message : String(error) });
    } finally {
      set({ themeSaving: false });
    }
  },
}));
