import { useReminderStore } from "../../stores/reminderStore";
import type { ReminderEvent } from "../../types/reminder";

const content = {
  eye: { title: "Eye Break", detail: "Look away from the screen for 20 seconds" },
  water: { title: "Drink some water", detail: "Take a moment to hydrate" },
  stand: { title: "Stand up", detail: "Move around for a little while" },
};

type Props = { event: ReminderEvent; onBack: () => void };

export function ReminderAlert({ event, onBack }: Props) {
  const snapshot = useReminderStore((state) => state.snapshot);
  const error = useReminderStore((state) => state.error);
  const complete = useReminderStore((state) => state.complete);
  const snooze = useReminderStore((state) => state.snooze);
  const rule = snapshot?.rules.find((item) => item.id === event.reminderId);
  const message = content[event.reminderId];

  async function act(kind: "complete" | "snooze") {
    const saved = kind === "complete" ? await complete(event.id) : await snooze(event.id);
    if (saved && !useReminderStore.getState().snapshot?.active.length) onBack();
  }

  return (
    <section className="reminder-alert" aria-label="Health reminder">
      <div className="panel-heading"><span>{rule?.icon} {message.title}</span><button type="button" aria-label="Close reminder" onClick={onBack}>×</button></div>
      <p>{message.detail}</p>
      {error && <p className="todo-error" role="alert">{error}</p>}
      <div className="reminder-actions">
        <button type="button" className="primary-action" onClick={() => void act("complete")}>✓ Done</button>
        <button type="button" onClick={() => void act("snooze")}>{rule?.snoozeMinutes ?? 10}m Later</button>
      </div>
    </section>
  );
}
