"use client";

import {
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";
interface EventDurationByDate {
  duration: number;
  date: string;
}
import { Event } from "../page";

const getTaskDurationByDay = (events: Event[]): EventDurationByDate[] => {
  const durationByDay: { [index: string]: EventDurationByDate } = {};
  events.forEach((taskEvent) => {
    const eventDate = taskEvent.time_stamp.toString().substring(0, 10);
    durationByDay[eventDate]
      ? (durationByDay[eventDate].duration += taskEvent.duration)
      : (durationByDay[eventDate] = {
          duration: taskEvent.duration,
          date: eventDate,
        });
  });
  return Object.values(durationByDay);
};

interface TimeChartProps {
  taskEvents: Event[];
}

const TimeChart = ({ taskEvents }: TimeChartProps) => {
  return (
    <LineChart width={600} height={200} data={getTaskDurationByDay(taskEvents)}>
      <XAxis dataKey="date" />
      <YAxis tick={false} />
      <Line type="monotone" dataKey={"duration"} stroke="#8884d8" />
    </LineChart>
  );
};

export default TimeChart;
