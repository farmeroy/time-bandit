#[derive(Debug)]
pub struct TaskWithEvents {
    pub task: Task,
    pub events: Option<Vec<Event>>,
}

#[derive(Debug)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: i32,
    pub task_id: i32,
    pub notes: Option<String>,
    pub time_stamp: String,
    pub duration: i32,
}

#[derive(Debug)]
pub struct EventWithTaskName {
    pub event: Event,
    pub task_name: String,
}
