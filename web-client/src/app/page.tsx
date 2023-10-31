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

interface Event {
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
    <div className="" key={task.task.id}>
      <div className="">
        <p>{task.task.name}</p>
        <div className="">Time Spent</div>
        <div className="">
          {new Date(
            task.events.reduce((acc, curr) => acc + curr.duration, 0) * 1000
          )
            .toISOString()
            .substring(11, 19)}
        </div>
        <div className="">
          <div className="">Number of Events</div>
          <div className="">{task.events.length}</div>
        </div>
      </div>
    </div>
  ));
  return <div className="w-full flex flex-col">{taskCards}</div>;
}
