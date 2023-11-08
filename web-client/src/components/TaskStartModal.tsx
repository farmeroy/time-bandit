import { TaskWithEvents } from "@/app/page";

const TaskStartCard = ({
  task,
  onCancel,
}: {
  task: TaskWithEvents;
  onCancel: () => void;
}) => {
  return (
    <div className="flex flex-col justify-between">
      <p className="text-lg">
        Lets work on: <span className="text-accent">{task.task.name}</span>
      </p>
      <form
        onSubmit={(event) => event.preventDefault()}
        className="flex flex-col"
      >
        <textarea
          className="textarea textarea-primary"
          placeholder="Add any notes here"
          name="event-name"
        />
        <div className="join py-2">
          <button className="btn btn-accent join-item">Start</button>
          <button className="btn btn-neutral join-item" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
};

export default TaskStartCard;
