import type { ReminderEvent, ReminderSnapshot } from "../types/reminder";

export function selectedReminderEvents(snapshot: ReminderSnapshot | null): ReminderEvent[] {
  if (!snapshot?.active.length) return [];
  if (snapshot.attentionVisible && snapshot.attentionEventIds.length) {
    const visible = snapshot.active.filter((event) => snapshot.attentionEventIds.includes(event.id));
    if (visible.length) return visible;
  }
  const first = snapshot.active[0];
  if (first.reminderId === "water") return [first];
  const partner = snapshot.active.find((event) =>
    event.id !== first.id && event.reminderId !== "water" && event.reminderId !== first.reminderId
      && event.triggeredAt === first.triggeredAt);
  return partner ? [first, partner].sort((a, b) => a.reminderId === "eye" ? -1 : b.reminderId === "eye" ? 1 : 0) : [first];
}
