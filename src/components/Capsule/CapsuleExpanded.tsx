import type { PointerEventHandler } from "react";
import { useTaskStore } from "../../stores/taskStore";
import { useReminderStore } from "../../stores/reminderStore";
import { TodoList } from "../Todo/TodoList";
import { ReminderCountdown } from "../Reminder/ReminderCountdown";

type Props = {
  onCollapse: () => void;
  onPointerDown: PointerEventHandler<HTMLElement>;
  onPointerMove: PointerEventHandler<HTMLElement>;
  onHistory: () => void;
  onSettings: () => void;
  onReminder: () => void;
};

export function CapsuleExpanded({ onCollapse, onPointerDown, onPointerMove, onHistory, onSettings, onReminder }: Props) {
  const tasks = useTaskStore((state) => state.tasks);
  const snapshot = useReminderStore((state) => state.snapshot);
  const completed = tasks.filter((task) => task.status === "completed").length;
  return (
    <section className="capsule-expanded" aria-label="Capsule today panel">
      <header className="capsule-header" onPointerDown={onPointerDown} onPointerMove={onPointerMove}>
        <span className="capsule-heading">TODAY</span>
        <span className="capsule-count">{completed} / {tasks.length}</span>
        <button className="capsule-collapse" type="button" aria-label="Collapse Capsule" onClick={onCollapse}>⌃</button>
      </header>
      {snapshot?.active.length ? <button className="reminder-banner" type="button" onClick={onReminder}>● {snapshot.active.length} break reminder{snapshot.active.length > 1 ? "s" : ""} ready →</button> : null}
      <TodoList />
      <footer className="capsule-footer">
        <div className="capsule-reminder-preview" aria-label="Next health reminders">
          {snapshot?.rules.map((rule) => <span key={rule.id}>{rule.icon} <ReminderCountdown nextAt={snapshot.next.find((item) => item.reminderId === rule.id)?.nextTriggerAt} /></span>)}
        </div>
        <div className="capsule-links"><button type="button" onClick={onHistory}>History</button><button type="button" aria-label="Settings" onClick={onSettings}>⚙</button></div>
      </footer>
    </section>
  );
}
