"use client";

import { TaskWithEvents } from "@/app/page";
import TimeChart from "./TimeChart";
import TaskStartCard from "./TaskStartModal";
import { useRouter } from "next/navigation";

interface TaskCardProps {
  task: TaskWithEvents;
}

const openModal = (id: string) => {
  const element = document.getElementById(id) as HTMLDialogElement | null;
  if (element != null) {
    element.showModal();
  }
};

const closeModal = (id: string) => {
  const element = document.getElementById(id) as HTMLDialogElement | null;
  if (element != null) {
    element.close();
  }
};

const TaskCard = ({ task }: TaskCardProps) => {
  const router = useRouter();
  return (
    <div className="w-full max-w-[800px] join join-vertical sm:join-horizontal justify-between p-4 my-2 rounded-xl border border-1">
      <div className=" join-item w-48">
        <p className="text-xl">{task.task.name}</p>
        <div className="flex sm:flex-col">
          <button
            className="m-1 btn btn-primary focus:ring-1"
            onClick={() => openModal(`task-start-modal-${task.task.id}`)}
          >
            Begin
          </button>
          <button
            onClick={() => router.push(`/details/${task.task.id}`)}
            className="m-1 btn btn-neutral focus:ring-1"
          >
            Details
          </button>
        </div>
        <dialog id={`task-start-modal-${task.task.id}`} className="modal">
          <div className="modal-box">
            <TaskStartCard
              task={task}
              onCancel={() => closeModal(`task-start-modal-${task.task.id}`)}
            />
          </div>
        </dialog>
      </div>
      <div className="join-item mx-auto  h-48 w-full sm:w-2/5 pr-5">
        <TimeChart taskEvents={task.events} />
      </div>
      <div className="w-full w-full md:w-1/3 p-2">
        <div className="join-item w-full stats stats-vertical border ">
          <div className="stat">
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
    </div>
  );
};

export default TaskCard;
