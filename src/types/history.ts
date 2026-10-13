import type { Task } from "./task";
import type { ReminderId } from "./reminder";

export type ReminderCount = {
  reminderId: ReminderId;
  completed: number;
  total: number;
  completionRate: number;
};

export type HistoryDay = {
  date: string;
  tasks: Task[];
  tasksCompleted: number;
  tasksTotal: number;
  taskCompletionRate: number;
  reminders: ReminderCount[];
};
