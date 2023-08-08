use crate::types::types::{Event, Task};
use rusqlite::{Connection, OptionalExtension, Result};

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
        print!("Before the task table");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
    )",
            (),
        )?;
        println!("Made the task table");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event (
                id INTEGER PRIMARY KEY,
                task_id INTEGER,
                notes TEXT,
                time_stamp STRING,
                duration STRING,
                FOREIGN KEY(task_id) REFERENCES task(id)
            )",
            (),
        )?;
        println!("Made the db");
        Ok(Store { connection: conn })
    }
    pub fn add_task(
        self,
        name: String,
        details: String,
        now: String,
        duration: String,
    ) -> Result<(), rusqlite::Error> {
        let task_id: Result<Option<i32>> = self
            .connection
            .query_row("SELECT id FROM task WHERE name = (?1)", [&name], |row| {
                row.get(0)
            })
            .optional();

        match task_id.unwrap() {
            Some(task_id) => {
                self.connection.execute(
                    "INSERT INTO event (task_id,notes,time_stamp,duration) VALUES (?1, ?2, ?3, ?4)",
                    (task_id, details, now, duration),
                )?;
            }

            None => println!("No task"),
        }

        Ok(())
    }
    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let stmt = &mut self
            .connection
            .prepare("SELECT id, name, details FROM task")?;
        let task_iter = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                name: row.get(1)?,
                details: row.get(2)?,
                // time_stamp: row.get(3)?,
                // duration: row.get(4)?,
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
