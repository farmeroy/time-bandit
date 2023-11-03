"use client";

import { TaskWithEvents } from "@/app/page";
import TimeChart from "./TimeChart";
import TaskStartCard from "./TaskStartModal";

interface TaskCardProps {
  task: TaskWithEvents;
}

const openModal = (id: string) => {
  const element = document.getElementById(id) as HTMLDialogElement | null;
  if (element != null) {
    element.showModal();
  }
};

const TaskCard = ({ task }: TaskCardProps) => {
  return (
    <div className="w-full max-w-[800px] join join-vertical sm:join-horizontal justify-between p-4 my-2 rounded-xl border border-1">
      <div className=" join-item w-48">
        <p className="text-xl">{task.task.name}</p>
        <div className="flex sm:flex-col">
          <button
            className="m-1 btn btn-primary"
            onClick={() => openModal(`task-start-modal-${task.task.id}`)}
          >
            Start
          </button>
          <dialog id={`task-start-modal-${task.task.id}`} className="modal">
            <div className="modal-box">
              <TaskStartCard task={task} />
            </div>
          </dialog>
          <button className="m-1 btn btn-neutral">Details</button>
        </div>
      </div>
      <div className="join-item  h-48 w-full md:w-72 lg:w-96">
        <TimeChart taskEvents={task.events} />
      </div>
      <div className="join-item stats stats-vertical w-68 border ">
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
  );
};

export default TaskCard;
