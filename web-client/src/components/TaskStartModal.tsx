import { TaskWithEvents } from "@/app/page";

const TaskStartCard = ({ task }: { task: TaskWithEvents }) => {
  return (
    <div>
      <p>{task.task.name}</p>
      <form>
        <label htmlFor="event-name">Event Name</label>
        <input name="event-name" type="text" />
      </form>
    </div>
  );
};

export default TaskStartCard;
