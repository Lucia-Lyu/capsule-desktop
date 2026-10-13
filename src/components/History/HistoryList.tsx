import { useEffect, useState } from "react";
import { useHistoryStore } from "../../stores/historyStore";
import { HistoryDetail } from "./HistoryDetail";

export function HistoryList({ onBack }: { onBack: () => void }) {
  const range = useHistoryStore((state) => state.range);
  const days = useHistoryStore((state) => state.days);
  const error = useHistoryStore((state) => state.error);
  const setRange = useHistoryStore((state) => state.setRange);
  const refresh = useHistoryStore((state) => state.refresh);
  const [selectedDate, setSelectedDate] = useState<string | null>(null);
  useEffect(() => { void refresh(); }, [refresh]);
  const selected = days.find((day) => day.date === selectedDate);
  if (selected) return <section className="secondary-panel"><HistoryDetail day={selected} onBack={() => setSelectedDate(null)} /></section>;

  const taskCompleted = days.reduce((sum, day) => sum + day.tasksCompleted, 0);
  const taskTotal = days.reduce((sum, day) => sum + day.tasksTotal, 0);
  const healthCompleted = days.reduce((sum, day) => sum + day.reminders.reduce((count, item) => count + item.completed, 0), 0);
  const healthTotal = days.reduce((sum, day) => sum + day.reminders.reduce((count, item) => count + item.total, 0), 0);
  const rates = (["eye", "water", "stand"] as const).map((id) => {
    const completed = days.reduce((sum, day) => sum + (day.reminders.find((count) => count.reminderId === id)?.completed ?? 0), 0);
    const total = days.reduce((sum, day) => sum + (day.reminders.find((count) => count.reminderId === id)?.total ?? 0), 0);
    return { id, rate: total ? Math.floor(completed * 100 / total) : 0 };
  });
  return (
    <section className="secondary-panel">
      <div className="panel-heading"><button type="button" onClick={onBack}>←</button><strong>History</strong></div>
      <div className="history-ranges">
        {([1, 7, 30] as const).map((value) => <button key={value} type="button" className={range === value ? "selected" : ""} onClick={() => { setSelectedDate(null); void setRange(value); }}>{value === 1 ? "Today" : `${value} days`}</button>)}
      </div>
      <div className="history-overview"><span>Tasks {taskCompleted}/{taskTotal}</span><span>Health {healthCompleted}/{healthTotal}</span></div>
      <div className="history-rates"><span>Tasks {taskTotal ? Math.floor(taskCompleted * 100 / taskTotal) : 0}%</span>{rates.map(({ id, rate }) => <span key={id}>{id === "eye" ? "Eye" : id === "water" ? "Water" : "Stand"} {rate}%</span>)}</div>
      <div className="panel-scroll history-list">
        {days.map((day) => {
          const healthDone = day.reminders.reduce((sum, count) => sum + count.completed, 0);
          const healthCount = day.reminders.reduce((sum, count) => sum + count.total, 0);
          return <button type="button" key={day.date} onClick={() => setSelectedDate(day.date)}><strong>{day.date}</strong><span>Tasks {day.tasksCompleted}/{day.tasksTotal}</span><span>Health {healthDone}/{healthCount}</span></button>;
        })}
        {error && <p className="todo-error" role="alert">{error}</p>}
      </div>
    </section>
  );
}
