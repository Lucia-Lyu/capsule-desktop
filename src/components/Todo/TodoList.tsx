import { useTaskStore } from "../../stores/taskStore";
import { AddTodo } from "./AddTodo";
import { TodoItem } from "./TodoItem";
import { TodoCarryover } from "./TodoCarryover";

export function TodoList() {
  const tasks = useTaskStore((state) => state.tasks);
  const error = useTaskStore((state) => state.error);
  return (
    <div className="todo-section">
      <TodoCarryover />
      {tasks.length === 0 ? <p className="todo-empty">Nothing planned today</p> : (
        <ul className="todo-list">
          {tasks.map((task, index) => (
            <TodoItem
              key={task.id}
              task={task}
              canMoveUp={index > 0 && tasks[index - 1].status === task.status}
              canMoveDown={index < tasks.length - 1 && tasks[index + 1].status === task.status}
            />
          ))}
        </ul>
      )}
      <AddTodo />
      {error && <p className="todo-error" role="alert">{error}</p>}
    </div>
  );
}
