export type ReminderId = "eye" | "water" | "stand";

export type ReminderRule = {
  id: ReminderId;
  name: string;
  icon: string;
  intervalMinutes: number;
  snoozeMinutes: number;
  enabled: boolean;
  updatedAt: string;
};

export type ReminderEvent = {
  id: string;
  reminderId: ReminderId;
  scheduledAt: string;
  triggeredAt: string | null;
  action: "triggered" | "completed" | "snoozed" | "skipped";
  completedAt: string | null;
  snoozeCount: number;
};

export type ReminderSnapshot = {
  rules: ReminderRule[];
  next: { reminderId: ReminderId; nextTriggerAt: string }[];
  active: ReminderEvent[];
  quietStart: string;
  quietEnd: string;
  attentionSound: boolean;
  attentionRepeat: boolean;
  attentionVisible: boolean;
};
