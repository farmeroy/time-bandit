import TaskCard from "@/components/TaskCard";

const getTaskData = async () => {
  const res = await fetch(`http://localhost:7878/task-events`, {
    cache: "no-store",
  });
  if (!res.ok) throw new Error("Failed to fetch tasks");
  return res.json();
};

interface Task {
  id: number;
  name: string;
  details?: string;
}

export interface Event {
  id: number;
  task_id: number;
  notes: string;
  time_stamp: Date;
  duration: number;
}

export interface TaskWithEvents {
  task: Task;
  events: Event[];
}

export default async function Home() {
  const tasks: TaskWithEvents[] = await getTaskData().then((res) => res);
  const taskCards = tasks.map((task) => (
    <TaskCard key={task.task.id} task={task} />
  ));
  return (
    <div className="flex flex-col place-items-center p-6 w-full">
      {taskCards}
    </div>
  );
}
