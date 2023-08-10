#[derive(Debug)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub details: Option<String>,
    pub events: Option<Vec<Event>>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: i32,
    pub task_id: i32,
    pub notes: Option<String>,
    pub time_stamp: String,
    pub duration: String,
}
