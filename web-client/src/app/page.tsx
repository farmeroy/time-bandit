import TimeChart from "./components/TimeChart";

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

interface TaskWithEvents {
  task: Task;
  events: Event[];
}

export default async function Home() {
  const tasks: TaskWithEvents[] = await getTaskData().then((res) => res);
  const taskCards = tasks.map((task) => (
    <div key={task.task.id} className="w-full p-2 m-2 flex border border-1">
      <div className="w-full">
        <p className="text-xl">{task.task.name}</p>
        <TimeChart taskEvents={task.events} />
      </div>
      <div className="stats stats-vertical m-3 border w-96 ">
        <div className="stat w-full">
          <p className="stat-title">Time Spent</p>
          <p className="stat-value">
            {new Date(
              task.events.reduce((acc, curr) => acc + curr.duration, 0) * 1000
            )
              .toISOString()
              .substring(11, 19)}
          </p>
        </div>
        <div className="stat">
          <p className="stat-title">Number of Events</p>
          <p className="stat-value">{task.events.length}</p>
        </div>
      </div>
    </div>
  ));
  return <div className="flex flex-wrap w-full">{taskCards}</div>;
}
