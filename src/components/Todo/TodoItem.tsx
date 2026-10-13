import { useTaskStore } from "../../stores/taskStore";
import type { Task } from "../../types/task";
import { formatFullDate, formatShortDate, taskCreatedAt } from "../../lib/date";

type Props = { task: Task; canMoveUp: boolean; canMoveDown: boolean };

export function TodoItem({ task, canMoveUp, canMoveDown }: Props) {
  const createdAt = taskCreatedAt(task);
  const toggle = useTaskStore((state) => state.toggle);
  const remove = useTaskStore((state) => state.remove);
  const move = useTaskStore((state) => state.move);
  const deferToTomorrow = useTaskStore((state) => state.deferToTomorrow);
  return (
    <li className={`todo-item ${task.status}`}>
      <button className="todo-check" type="button" disabled={Boolean(task.carriedToDate)} title={task.carriedToDate ? `Continued ${task.carriedToDate}` : undefined} aria-label={`${task.status === "completed" ? "Uncomplete" : "Complete"} ${task.title}`} onClick={() => void toggle(task)}>
        {task.status === "completed" ? "✓" : ""}
      </button>
      <span className="todo-title" title={`Created ${formatFullDate(createdAt)}${task.carriedToDate ? ` · Continued ${task.carriedToDate}` : ""}`}>
        {task.originCreatedAt && <span aria-hidden="true">↪ </span>}{task.title}
      </span>
      <time className="todo-created" dateTime={createdAt} aria-label={`Created ${formatFullDate(createdAt)}`}>{formatShortDate(createdAt)}</time>
      {task.carriedToDate && <span className="todo-deferred" title={`Continued ${task.carriedToDate}`}>Tomorrow</span>}
      <span className="todo-actions">
        {task.status === "pending" && !task.carriedToDate && <button type="button" title="Continue tomorrow" aria-label={`Continue ${task.title} tomorrow`} onClick={() => void deferToTomorrow(task.id)}>↪</button>}
        <button type="button" aria-label={`Move ${task.title} up`} disabled={!canMoveUp} onClick={() => void move(task.id, -1)}>↑</button>
        <button type="button" aria-label={`Move ${task.title} down`} disabled={!canMoveDown} onClick={() => void move(task.id, 1)}>↓</button>
        <button type="button" aria-label={`Delete ${task.title}`} onClick={() => void remove(task.id)}>×</button>
      </span>
    </li>
  );
}
