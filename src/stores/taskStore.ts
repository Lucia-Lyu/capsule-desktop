import { isTauri } from "@tauri-apps/api/core";
import { create } from "zustand";
import { api } from "../lib/tauri";
import type { Task } from "../types/task";

type TaskState = {
  tasks: Task[];
  carryCandidates: Task[];
  error: string | null;
  refresh: () => Promise<void>;
  add: (title: string) => Promise<boolean>;
  toggle: (task: Task) => Promise<void>;
  remove: (id: string) => Promise<void>;
  move: (id: string, direction: -1 | 1) => Promise<void>;
  carry: (ids: string[]) => Promise<boolean>;
  deferToTomorrow: (id: string) => Promise<void>;
};

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useTaskStore = create<TaskState>((set, get) => ({
  tasks: [],
  carryCandidates: [],
  error: null,
  refresh: async () => {
    if (!isTauri()) return;
    try {
      const [tasks, carryCandidates] = await Promise.all([api.todayTasks(), api.carryCandidates()]);
      set({ tasks, carryCandidates, error: null });
    } catch (error) {
      set({ error: message(error) });
    }
  },
  add: async (title) => {
    try {
      await api.createTask(title);
      await get().refresh();
      return true;
    } catch (error) {
      set({ error: message(error) });
      return false;
    }
  },
  toggle: async (task) => {
    try {
      await api.setTaskCompleted(task.id, task.status !== "completed");
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    }
  },
  remove: async (id) => {
    try {
      await api.deleteTask(id);
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    }
  },
  move: async (id, direction) => {
    try {
      await api.moveTask(id, direction);
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    }
  },
  carry: async (ids) => {
    try {
      await api.carryTasks(ids);
      await get().refresh();
      return true;
    } catch (error) {
      await get().refresh();
      set({ error: message(error) });
      return false;
    }
  },
  deferToTomorrow: async (id) => {
    try {
      await api.deferTaskToTomorrow(id);
      await get().refresh();
    } catch (error) {
      set({ error: message(error) });
    }
  },
}));
