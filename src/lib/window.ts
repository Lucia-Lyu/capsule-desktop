import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export async function resizeCapsule(mode: string): Promise<void> {
  if (isTauri()) {
    await invoke("set_capsule_mode", { mode });
  }
}

export async function startCapsuleDrag(): Promise<void> {
  if (isTauri()) {
    await getCurrentWindow().startDragging();
  }
}

export async function onCapsuleFocusChanged(
  callback: (focused: boolean) => void,
): Promise<() => void> {
  if (!isTauri()) return () => {};
  return getCurrentWindow().onFocusChanged(({ payload }) => callback(payload));
}
