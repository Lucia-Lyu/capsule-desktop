import type { MouseEventHandler, PointerEventHandler } from "react";
import type { ReminderRule } from "../../types/reminder";
import { ReminderCountdown } from "../Reminder/ReminderCountdown";

type Props = {
  onClick: MouseEventHandler<HTMLButtonElement>;
  onPointerDown: PointerEventHandler<HTMLButtonElement>;
  onPointerMove: PointerEventHandler<HTMLButtonElement>;
  completed: number;
  total: number;
  rule?: ReminderRule;
  nextAt?: string;
  alert: boolean;
};

export function CapsuleCollapsed({ onClick, onPointerDown, onPointerMove, completed, total, rule, nextAt, alert }: Props) {
  return (
    <button
      className="capsule-collapsed"
      type="button"
      aria-label="Open Capsule"
      onClick={onClick}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
    >
      <span className={`capsule-next ${alert ? "is-alert" : ""}`}><span aria-hidden="true">{rule?.icon ?? "👁"}</span>{alert ? <span>Break</span> : <ReminderCountdown nextAt={nextAt} />}</span>
      <span className="capsule-divider" aria-hidden="true" />
      <span className="capsule-progress">{completed}/{total}</span>
    </button>
  );
}
