import { useEffect, useState } from "react";
import { useReminderStore } from "../../stores/reminderStore";
import type { ReminderRule } from "../../types/reminder";
import { useAppStore } from "../../stores/appStore";
import { capsuleThemes, isCapsuleTheme } from "../../types/theme";

function RuleSettings({ rule }: { rule: ReminderRule }) {
  const update = useReminderStore((state) => state.updateRule);
  const [interval, setIntervalMinutes] = useState(String(rule.intervalMinutes));
  const [snooze, setSnoozeMinutes] = useState(String(rule.snoozeMinutes));
  useEffect(() => { setIntervalMinutes(String(rule.intervalMinutes)); setSnoozeMinutes(String(rule.snoozeMinutes)); }, [rule.intervalMinutes, rule.snoozeMinutes]);
  function save(nextInterval = interval, nextSnooze = snooze) {
    void update(rule, rule.enabled, Number(nextInterval), Number(nextSnooze));
  }
  return (
    <div className="settings-rule">
      <div className="settings-rule-title"><span>{rule.icon} {rule.name}</span><label><span>Enabled</span><input type="checkbox" checked={rule.enabled} onChange={(event) => void update(rule, event.target.checked, rule.intervalMinutes, rule.snoozeMinutes)} /></label></div>
      <label>Every <input aria-label={`${rule.name} interval in minutes`} type="number" min={1} max={240} value={interval} onChange={(event) => setIntervalMinutes(event.target.value)} onBlur={() => save()} onKeyDown={(event) => { if (event.key === "Enter") event.currentTarget.blur(); }} /> min</label>
      <label>Snooze <input aria-label={`${rule.name} snooze in minutes`} type="number" min={1} max={120} value={snooze} onChange={(event) => setSnoozeMinutes(event.target.value)} onBlur={() => save()} onKeyDown={(event) => { if (event.key === "Enter") event.currentTarget.blur(); }} /> min</label>
    </div>
  );
}

export function SettingsPanel({ onBack }: { onBack: () => void }) {
  const theme = useAppStore((state) => state.theme);
  const setTheme = useAppStore((state) => state.setTheme);
  const themeError = useAppStore((state) => state.themeError);
  const themeSaving = useAppStore((state) => state.themeSaving);
  const snapshot = useReminderStore((state) => state.snapshot);
  const error = useReminderStore((state) => state.error);
  const updateQuiet = useReminderStore((state) => state.updateQuiet);
  const updateAttention = useReminderStore((state) => state.updateAttention);
  const [start, setStart] = useState(snapshot?.quietStart ?? "22:00");
  const [end, setEnd] = useState(snapshot?.quietEnd ?? "08:00");
  useEffect(() => { if (snapshot) { setStart(snapshot.quietStart); setEnd(snapshot.quietEnd); } }, [snapshot?.quietStart, snapshot?.quietEnd]);
  return (
    <section className="secondary-panel">
      <div className="panel-heading"><button type="button" onClick={onBack}>←</button><strong>Settings</strong></div>
      <div className="panel-scroll">
        <h2>CAPSULE COLOR</h2>
        <label className="theme-select">
          <span className={`theme-swatch theme-swatch-${theme}`} aria-hidden="true" />
          <select
            aria-label="Capsule color"
            value={theme}
            disabled={themeSaving}
            onChange={(event) => {
              if (isCapsuleTheme(event.target.value)) void setTheme(event.target.value);
            }}
          >
            {capsuleThemes.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}
          </select>
          <span className="theme-select-arrow" aria-hidden="true">⌄</span>
        </label>
        {themeError && <p className="todo-error" role="alert">{themeError}</p>}
        <h2>REMINDERS</h2>
        {snapshot?.rules.map((rule) => <RuleSettings key={rule.id} rule={rule} />)}
        <h2>ATTENTION</h2>
        <div className="attention-settings">
          <label><span>Play sound</span><input type="checkbox" checked={snapshot?.attentionSound ?? true} onChange={(event) => void updateAttention(event.target.checked, snapshot?.attentionRepeat ?? true)} /></label>
          <label><span>Repeat after 90 seconds</span><input type="checkbox" checked={snapshot?.attentionRepeat ?? true} onChange={(event) => void updateAttention(snapshot?.attentionSound ?? true, event.target.checked)} /></label>
        </div>
        <h2>QUIET HOURS</h2>
        <div className="quiet-hours"><label>Start <input type="time" value={start} onChange={(event) => setStart(event.target.value)} onBlur={() => void updateQuiet(start, end)} /></label><label>End <input type="time" value={end} onChange={(event) => setEnd(event.target.value)} onBlur={() => void updateQuiet(start, end)} /></label></div>
        {error && <p className="todo-error" role="alert">{error}</p>}
      </div>
    </section>
  );
}
