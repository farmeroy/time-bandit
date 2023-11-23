import { TaskWithEvents } from "@/app/page";

const getTaskDetails = async (id: string): Promise<TaskWithEvents> => {
  const res = await fetch(`http://localhost:7878/tasks/${id}`, {
    cache: "no-store",
  });
  if (!res.ok) throw new Error("Failed to fetch tasks");
  return res.json();
};

const DetailsPage = async ({ params }: { params: { slug: string } }) => {
  const { task, events } = await getTaskDetails(params.slug);
  return (
    <div>
      <h2>{task.name}</h2>
      <table className="table table-pin-rows">
        <thead>
          <tr>
            <th>Date</th>
            <th>Duration</th>
            <th>Notes</th>
          </tr>
        </thead>
        <tbody>
          {events.map((event) => (
            <tr key={event.id}>
              <td>{event.time_stamp.toString().substring(0, 10)}</td>
              <td>{event.duration}</td>
              <td>{event.notes}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default DetailsPage;
