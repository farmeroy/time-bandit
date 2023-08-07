use crate::types::types::Task;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct Store {
    connection: Connection,
}

// create an event type
// so that each task can have multiple events
// id -> task id
// duration
// timestamp

impl Store {
    pub fn new(db_url: &str) -> Result<Self> {
        let conn = Connection::open(db_url)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            details TEXT,
            time_stamp TEXT NOT NULL,
            duration TEXT NOT NULL
    )",
            (),
        )?;
        Ok(Store { connection: conn })
    }
    pub fn add_task(self, task: Task) -> Result<(), rusqlite::Error> {
        self.connection.execute(
            "INSERT INTO task (name, details, time_stamp, duration) VALUES (?1, ?2, ?3, ?4)",
            (&task.name, &task.details, &task.time_stamp, &task.duration),
        )?;
        Ok(())
    }
    pub fn get_tasks(self) -> Result<Vec<Task>> {
        let stmt = &mut self
            .connection
            .prepare("SELECT id, name, details, time_stamp, duration FROM task")?;
        let task_iter = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                name: row.get(1)?,
                details: row.get(2)?,
                time_stamp: row.get(3)?,
                duration: row.get(4)?,
            })
        })?;
        let mut tasks = vec![];
        for task in task_iter {
            let task = task.unwrap();
            tasks.push(task)
        }
        Ok(tasks)
    }
}
