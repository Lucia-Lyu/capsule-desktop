import { useState } from "react";
import { useTaskStore } from "../../stores/taskStore";

export function AddTodo() {
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState("");
  const add = useTaskStore((state) => state.add);

  async function submit() {
    const value = title.trim();
    if (!value) {
      setEditing(false);
      return;
    }
    if (await add(value)) {
      setTitle("");
      setEditing(false);
    }
  }

  return editing ? (
    <input
      autoFocus
      className="todo-input"
      aria-label="New task"
      maxLength={240}
      placeholder="What needs doing?"
      value={title}
      onChange={(event) => setTitle(event.target.value)}
      onKeyDown={(event) => {
        if (event.key === "Enter") void submit();
        if (event.key === "Escape") { setTitle(""); setEditing(false); }
      }}
      onBlur={() => { if (!title.trim()) setEditing(false); }}
    />
  ) : (
    <button className="todo-add" type="button" onClick={() => setEditing(true)}>＋ Add task</button>
  );
}
