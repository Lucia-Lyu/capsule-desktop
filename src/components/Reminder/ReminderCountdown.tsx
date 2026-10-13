import { useEffect, useState } from "react";

export function ReminderCountdown({ nextAt }: { nextAt?: string }) {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1_000);
    return () => window.clearInterval(timer);
  }, []);
  if (!nextAt) return <span>—</span>;
  const minutes = Math.max(0, Math.ceil((new Date(nextAt).getTime() - now) / 60_000));
  return <span>{minutes}m</span>;
}
