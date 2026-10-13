import type { HistoryDay } from "../../types/history";
import { formatFullDate, formatLocalDay, taskCreatedAt } from "../../lib/date";

const labels = { eye: "Eye", water: "Water", stand: "Stand" };

export function HistoryDetail({ day, onBack }: { day: HistoryDay; onBack: () => void }) {
  return (
    <>
      <div className="panel-heading"><button type="button" onClick={onBack}>←</button><strong>{day.date}</strong></div>
      <div className="panel-scroll history-detail">
        {day.tasks.length ? <ul>{day.tasks.map((task) => <li key={task.id} className={task.status}>
          <span className="history-task-title">{task.status === "completed" ? "✓" : "□"} {task.title}</span>
          <small className="history-task-date">Created {formatFullDate(taskCreatedAt(task))}{task.carriedToDate ? ` · Continued ${formatLocalDay(task.carriedToDate)}` : ""}</small>
        </li>)}</ul> : <p>No tasks recorded</p>}
        <div className="history-stats"><span>Tasks</span><strong>{day.tasksCompleted} / {day.tasksTotal}</strong><small>{day.taskCompletionRate}%</small></div>
        {day.reminders.map((count) => <div className="history-stats" key={count.reminderId}><span>{labels[count.reminderId]}</span><strong>{count.completed} / {count.total}</strong><small>{count.completionRate}%</small></div>)}
      </div>
    </>
  );
}
