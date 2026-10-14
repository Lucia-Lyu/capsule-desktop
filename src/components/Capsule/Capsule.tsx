import { useEffect, useRef } from "react";
import type { MouseEvent as ReactMouseEvent, PointerEvent as ReactPointerEvent } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onCapsuleFocusChanged, resizeCapsule, startCapsuleDrag } from "../../lib/window";
import { selectedReminderEvents } from "../../lib/reminderGroup";
import { useAppStore } from "../../stores/appStore";
import type { CapsuleMode } from "../../stores/appStore";
import { useTaskStore } from "../../stores/taskStore";
import { useReminderStore } from "../../stores/reminderStore";
import { useHistoryStore } from "../../stores/historyStore";
import { CapsuleCollapsed } from "./CapsuleCollapsed";
import { CapsuleExpanded } from "./CapsuleExpanded";
import { ReminderAlert } from "../Reminder/ReminderAlert";
import { SettingsPanel } from "../Settings/SettingsPanel";
import { HistoryList } from "../History/HistoryList";

type PointerOrigin = { x: number; y: number };

function safelyUnlisten(stop: () => void) {
  void Promise.resolve().then(stop).catch(() => {});
}

export function Capsule() {
  const mode = useAppStore((state) => state.capsuleMode);
  const setMode = useAppStore((state) => state.setCapsuleMode);
  const tasks = useTaskStore((state) => state.tasks);
  const refreshTasks = useTaskStore((state) => state.refresh);
  const snapshot = useReminderStore((state) => state.snapshot);
  const selectedReminders = selectedReminderEvents(snapshot);
  const refreshReminders = useReminderStore((state) => state.refresh);
  const refreshHistory = useHistoryStore((state) => state.refresh);
  const completed = tasks.filter((task) => task.status === "completed").length;
  const pointerOrigin = useRef<PointerOrigin | null>(null);
  const dragged = useRef(false);
  const transitioning = useRef(false);

  async function changeMode(nextMode: CapsuleMode) {
    if (transitioning.current) return;
    transitioning.current = true;
    try {
      await resizeCapsule(nextMode, nextMode === "reminder" && selectedReminderEvents(useReminderStore.getState().snapshot).length > 1);
      setMode(nextMode);
    } catch (error) {
      console.error("Could not resize Capsule", error);
    } finally {
      transitioning.current = false;
    }
  }

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    onCapsuleFocusChanged((focused) => {
      if (!focused && !useAppStore.getState().attentionVisible && useAppStore.getState().capsuleMode !== "collapsed") {
        void changeMode("collapsed");
      }
    }).then((stop) => {
      if (disposed) safelyUnlisten(stop);
      else unlisten = stop;
    }).catch((error) => console.error("Could not observe Capsule focus", error));
    return () => {
      disposed = true;
      if (unlisten) safelyUnlisten(unlisten);
    };
  }, []);

  useEffect(() => {
    void refreshTasks();
    void refreshReminders().then(() => {
      if (useReminderStore.getState().snapshot?.attentionVisible) {
        useAppStore.getState().setAttentionVisible(true);
        useAppStore.getState().setCapsuleMode("reminder");
      }
    });
    let day = new Date().toLocaleDateString("en-CA");
    const interval = window.setInterval(() => {
      const current = new Date().toLocaleDateString("en-CA");
      if (day !== current) {
        day = current;
        void refreshTasks();
      }
    }, 30_000);
    let disposed = false;
    let unlisten: (() => void) | undefined;
    let unlistenReminder: (() => void) | undefined;
    let unlistenDay: (() => void) | undefined;
    let unlistenAttention: (() => void) | undefined;
    if (isTauri()) {
      listen("task://updated", () => { void refreshTasks(); if (useAppStore.getState().capsuleMode === "history") void refreshHistory(); }).then((stop) => {
        if (disposed) safelyUnlisten(stop); else unlisten = stop;
      }).catch((error) => console.error("Could not observe task changes", error));
      listen("reminder://updated", () => { void refreshReminders(); if (useAppStore.getState().capsuleMode === "history") void refreshHistory(); }).then((stop) => {
        if (disposed) safelyUnlisten(stop); else unlistenReminder = stop;
      }).catch((error) => console.error("Could not observe reminders", error));
      listen("day://changed", () => { void refreshTasks(); if (useAppStore.getState().capsuleMode === "history") void refreshHistory(); }).then((stop) => {
        if (disposed) safelyUnlisten(stop); else unlistenDay = stop;
      }).catch((error) => console.error("Could not observe date rollover", error));
      listen<boolean>("reminder://attention", ({ payload }) => {
        useAppStore.getState().setAttentionVisible(payload);
        useAppStore.getState().setCapsuleMode(payload ? "reminder" : "collapsed");
        void refreshReminders();
      }).then((stop) => {
        if (disposed) safelyUnlisten(stop); else unlistenAttention = stop;
      }).catch((error) => console.error("Could not observe reminder attention", error));
    }
    return () => {
      disposed = true;
      window.clearInterval(interval);
      for (const stop of [unlisten, unlistenReminder, unlistenDay, unlistenAttention]) if (stop) safelyUnlisten(stop);
    };
  }, [refreshTasks, refreshReminders, refreshHistory]);

  function onPointerDown(event: ReactPointerEvent<HTMLElement>) {
    if (event.button !== 0) return;
    pointerOrigin.current = { x: event.screenX, y: event.screenY };
    dragged.current = false;
  }

  function onPointerMove(event: ReactPointerEvent<HTMLElement>) {
    const origin = pointerOrigin.current;
    if (!origin || dragged.current || event.buttons !== 1) return;
    if (Math.hypot(event.screenX - origin.x, event.screenY - origin.y) < 5) return;
    dragged.current = true;
    pointerOrigin.current = null;
    void startCapsuleDrag().catch((error) => console.error("Could not drag Capsule", error));
  }

  function onCollapsedClick(event: ReactMouseEvent<HTMLButtonElement>) {
    pointerOrigin.current = null;
    // Keyboard and accessibility activation does not begin with pointer down.
    if (event.detail === 0 || !dragged.current) void changeMode(snapshot?.active.length ? "reminder" : "expanded");
  }

  if (mode === "collapsed") {
    const active = snapshot?.active[0];
    const next = snapshot?.next[0];
    const id = active?.reminderId ?? next?.reminderId;
    return <CapsuleCollapsed onClick={onCollapsedClick} onPointerDown={onPointerDown} onPointerMove={onPointerMove} completed={completed} total={tasks.length} rule={snapshot?.rules.find((rule) => rule.id === id)} nextAt={next?.nextTriggerAt} alert={Boolean(active)} />;
  }
  if (mode === "reminder") {
    return selectedReminders.length ? <ReminderAlert events={selectedReminders} onBack={() => void changeMode("collapsed")} /> : <CapsuleExpanded onCollapse={() => void changeMode("collapsed")} onPointerDown={onPointerDown} onPointerMove={onPointerMove} onHistory={() => void changeMode("history")} onSettings={() => void changeMode("settings")} onReminder={() => void changeMode("reminder")} />;
  }
  if (mode === "settings") return <SettingsPanel onBack={() => void changeMode("expanded")} />;
  if (mode === "history") return <HistoryList onBack={() => void changeMode("expanded")} />;
  return <CapsuleExpanded onCollapse={() => void changeMode("collapsed")} onPointerDown={onPointerDown} onPointerMove={onPointerMove} onHistory={() => void changeMode("history")} onSettings={() => void changeMode("settings")} onReminder={() => void changeMode("reminder")} />;
}
