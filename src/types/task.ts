export type Task = {
  id: string;
  title: string;
  taskDate: string;
  status: "pending" | "completed";
  sortOrder: number;
  createdAt: string;
  completedAt: string | null;
  originCreatedAt: string | null;
  carriedToDate: string | null;
};
