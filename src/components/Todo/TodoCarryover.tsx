import { useState } from "react";
import { formatFullDate, formatLocalDay, taskCreatedAt } from "../../lib/date";
import { useTaskStore } from "../../stores/taskStore";

export function TodoCarryover() {
  const candidates = useTaskStore((state) => state.carryCandidates);
  const carry = useTaskStore((state) => state.carry);
  const [expanded, setExpanded] = useState(false);
  const [busy, setBusy] = useState(false);

  if (candidates.length === 0) return null;

  async function bring(ids: string[]) {
    setBusy(true);
    await carry(ids);
    setBusy(false);
  }

  return (
    <div className="todo-carryover">
      <div className="todo-carryover-heading">
        <button type="button" onClick={() => setExpanded(!expanded)} aria-expanded={expanded}>
          ↪ {candidates.length} unfinished from earlier {expanded ? "⌃" : "⌄"}
        </button>
        <button type="button" disabled={busy} onClick={() => void bring(candidates.map((task) => task.id))}>Bring all</button>
      </div>
      {expanded && <div className="todo-carryover-list">
        {candidates.map((task) => <div className="todo-carryover-row" key={task.id}>
          <span title={`Created ${formatFullDate(taskCreatedAt(task))}`}>
            <strong>{task.title}</strong>
            <small>{formatLocalDay(task.taskDate)} · Created {formatFullDate(taskCreatedAt(task))}</small>
          </span>
          <button type="button" aria-label={`Bring ${task.title} to today`} disabled={busy} onClick={() => void bring([task.id])}>Bring</button>
        </div>)}
      </div>}
    </div>
  );
}
