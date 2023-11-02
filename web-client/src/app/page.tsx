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
    <div
      key={task.task.id}
      className="w-full flex join justify-between p-2 m-4 flex rounded-xl border border-1"
    >
      <div className="p-4 join-item flex flex-col">
        <p className="text-xl">{task.task.name}</p>
        <button className="m-1 btn btn-primary">Start</button>
        <button className="m-1 btn btn-neutral">Details</button>
      </div>
      <div className="join-item h-48 flex flex-col w-96 mt-8">
        <TimeChart taskEvents={task.events} />
      </div>
      <div className="join-item stats stats-vertical w-56 m-3 border ">
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
  return <div className="flex p-12 flex-wrap w-full">{taskCards}</div>;
}
