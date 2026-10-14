import { useState } from "react";
import { useReminderStore } from "../../stores/reminderStore";
import type { ReminderEvent, ReminderId } from "../../types/reminder";

const content: Record<ReminderId, { title: string; detail: string; short: string }> = {
  eye: { title: "Eye Break", detail: "Look away from the screen for 20 seconds", short: "Rest your eyes" },
  water: { title: "Drink some water", detail: "Take a moment to hydrate", short: "Drink water" },
  stand: { title: "Stand up", detail: "Move around for a little while", short: "Stand up and move" },
};

type Props = { events: ReminderEvent[]; onBack: () => void };

export function ReminderAlert({ events, onBack }: Props) {
  const snapshot = useReminderStore((state) => state.snapshot);
  const error = useReminderStore((state) => state.error);
  const complete = useReminderStore((state) => state.complete);
  const snooze = useReminderStore((state) => state.snooze);
  const [busy, setBusy] = useState(false);
  const grouped = events.length > 1;
  const first = events[0];
  const firstRule = snapshot?.rules.find((rule) => rule.id === first.reminderId);
  const snoozeMinutes = Math.max(...events.map((event) => snapshot?.rules.find((rule) => rule.id === event.reminderId)?.snoozeMinutes ?? 10));

  async function act(kind: "complete" | "snooze") {
    if (busy) return;
    setBusy(true);
    const ids = events.map((event) => event.id);
    const saved = kind === "complete" ? await complete(ids) : await snooze(ids);
    setBusy(false);
    if (saved && !useReminderStore.getState().snapshot?.active.length) onBack();
  }

  return (
    <section className={`reminder-alert ${grouped ? "is-grouped" : ""}`} aria-label="Health reminder">
      <div className="panel-heading"><span>{grouped ? "Time for a short break" : `${firstRule?.icon ?? ""} ${content[first.reminderId].title}`}</span><button type="button" aria-label="Close reminder" onClick={onBack}>×</button></div>
      {grouped ? <ul className="reminder-group-items">{events.map((event) => {
        const rule = snapshot?.rules.find((item) => item.id === event.reminderId);
        return <li key={event.id}><span aria-hidden="true">{rule?.icon}</span>{content[event.reminderId].short}</li>;
      })}</ul> : <p>{content[first.reminderId].detail}</p>}
      {error && <p className="todo-error" role="alert">{error}</p>}
      <div className="reminder-actions">
        <button type="button" className="primary-action" disabled={busy} onClick={() => void act("complete")}>✓ {grouped ? "Done all" : "Done"}</button>
        <button type="button" disabled={busy} onClick={() => void act("snooze")}>{snoozeMinutes}m Later</button>
      </div>
    </section>
  );
}
