use crate::types::types::{Event, Task};
use rusqlite::{params, Connection, OptionalExtension, Result};

#[derive(Debug)]
pub struct Store {
    connection: Connection,
}

fn get_task_id_by_name(conn: &Connection, task_name: &str) -> Result<Option<i32>> {
    let mut stmt = conn.prepare("SELECT id FROM task WHERE name = ?")?;
    let task_id: Result<Option<i32>> = stmt.query_row(&[task_name], |row| row.get(0));
    Ok(task_id.unwrap_or(None))
}

fn create_task(conn: &Connection, task_name: &str) -> Result<Task> {
    conn.execute("INSERT INTO task (name) VALUES (?)", &[task_name])?;
    let task_id = conn.last_insert_rowid() as i32;

    let new_task = Task {
        id: task_id,
        name: task_name.to_string(),
        details: Some(String::from("")),
    };

    Ok(new_task)
}

fn create_event(
    conn: &Connection,
    task_id: &i32,
    notes: &str,
    now: &str,
    duration: &str,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO event (task_id, notes, time_stamp, duration) VALUES (?1, ?2, ?3, ?4) ",
    )?;
    stmt.execute(params![task_id, notes, now, duration])?;
    Ok(())
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
            details TEXT
    )",
            (),
        )?;
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
        Ok(Store { connection: conn })
    }
    pub fn add_task(
        &self,
        name: String,
        details: String,
        now: String,
        duration: String,
    ) -> Result<(), rusqlite::Error> {
        if let Some(task_id) = get_task_id_by_name(&self.connection, &name)? {
            create_event(&self.connection, &task_id, &details, &now, &duration)?;
        } else {
            let new_task = create_task(&self.connection, &name)?;
            create_event(&self.connection, &new_task.id, &details, &now, &duration)?;
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
            })
        })?;
        let mut tasks = vec![];
        for task in task_iter {
            let task = task.unwrap();
            tasks.push(task)
        }
        Ok(tasks)
    }
    pub fn get_events(&self) -> Result<Vec<Event>> {
        let stmt = &mut self
            .connection
            .prepare("SELECT id, task_id, notes, time_stamp, duration FROM event")?;
        let event_iter = stmt.query_map([], |row| {
            Ok(Event {
                id: row.get(0)?,
                task_id: row.get(1)?,
                notes: row.get(2)?,
                time_stamp: row.get(3)?,
                duration: row.get(4)?,
            })
        })?;
        let mut events = vec![];
        for event in event_iter {
            let event = event.unwrap();
            events.push(event)
        }
        Ok(events)
    }
}
