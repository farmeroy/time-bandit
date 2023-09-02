use crate::types::types::{Event, EventWithTaskName, Task, TaskWithEvents};
use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct Store {
    connection: Connection,
}

fn get_task_id_by_name(conn: &Connection, task_name: &str) -> Result<Option<i32>> {
    let mut stmt = conn.prepare("SELECT id FROM task WHERE name = ?")?;
    let task_id: Result<Option<i32>> = stmt.query_row(&[task_name], |row| row.get(0));
    Ok(task_id.unwrap_or(None))
}

fn create_task(conn: &Connection, task_name: &str, details: &str) -> Result<Task> {
    conn.execute(
        "INSERT INTO task (name, details) VALUES (?1, ?2)",
        &[task_name, details],
    )?;
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
    duration: &u64,
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
                duration INTEGER,
                FOREIGN KEY(task_id) REFERENCES task(id)
            )",
            (),
        )?;
        Ok(Store { connection: conn })
    }
    /// Add an event associated with a task,
    /// creating a new task if it doesn't exist
    pub fn add_task_event(
        &self,
        name: String,
        details: String,
        now: String,
        duration: u64,
    ) -> Result<(), rusqlite::Error> {
        if let Some(task_id) = get_task_id_by_name(&self.connection, &name)? {
            create_event(&self.connection, &task_id, &details, &now, &duration)?;
        } else {
            let new_task = create_task(&self.connection, &name, &details)?;
            create_event(&self.connection, &new_task.id, &details, &now, &duration)?;
        }
        Ok(())
    }
    /// Fetch all tasks
    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let stmt = &mut self.connection.prepare(
            "
            SELECT * FROM task
            ",
        )?;

        let task_iter = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get("id")?,
                name: row.get("name")?,
                details: row.get("details")?,
            })
        })?;
        let mut tasks = vec![];
        for task in task_iter {
            let task = task.unwrap();
            tasks.push(task)
        }
        Ok(tasks)
    }
    /// Fetch all tasks together with their associated events
    pub fn get_tasks_with_events(&self) -> Result<Vec<TaskWithEvents>> {
        let stmt = &mut self.connection.prepare(
            "
        SELECT
            task.id AS task_id,
            task.name AS task_name,
            task.details AS task_details,
            event.id AS event_id,
            event.task_id AS event_task_id,
            event.notes AS event_notes,
            event.time_stamp AS event_time_stamp,
            event.duration AS event_duration
        FROM
            task
        LEFT JOIN
            event ON task.id = event.task_id
    ",
        )?;

        let mut rows = stmt.query([])?;
        let mut tasks_with_events: Vec<TaskWithEvents> = Vec::new();
        let mut current_task: Option<TaskWithEvents> = None;

        while let Some(row) = rows.next()? {
            let task_id: i32 = row.get("task_id")?;
            if let Some(task) = current_task.take() {
                tasks_with_events.push(task);
            }

            let task_name: String = row.get("task_name")?;
            let task_details: Option<String> = row.get("task_details")?;
            let event_id: Option<i32> = row.get("event_id")?;
            let mut events: Vec<Event> = Vec::new();

            if let Some(id) = event_id {
                let event_task_id: i32 = row.get("event_task_id")?;
                let event_notes: Option<String> = row.get("event_notes")?;
                let event_time_stamp: String = row.get("event_time_stamp")?;
                let event_duration: i32 = row.get("event_duration")?;

                events.push(Event {
                    id,
                    task_id: event_task_id,
                    notes: event_notes,
                    time_stamp: event_time_stamp,
                    duration: event_duration,
                });
            }
            current_task = Some(TaskWithEvents {
                task: Task {
                    id: task_id,
                    name: task_name,
                    details: task_details,
                },
                events: Some(events),
            });
            if let Some(task) = current_task.take() {
                tasks_with_events.push(task);
            }
        }

        Ok(tasks_with_events)
    }
    pub fn get_events_by_task(&self, task_name: String) -> Result<Vec<EventWithTaskName>> {
        let stmt = &mut self.connection.prepare(
            "SELECT 
                event.id AS event_id,
                event.task_id AS event_task_id,
                event.notes AS event_notes,
                event.time_stamp AS event_time_stamp,
                event.duration AS event_duration,
                task.name AS task_name
                FROM 
                    event
                LEFT JOIN 
                    task ON event.task_id = task.id
                    WHERE task.name = $1
            ",
        )?;
        let event_iter = stmt.query_map([task_name], |row| {
            Ok(EventWithTaskName {
                event: Event {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    notes: row.get(2)?,
                    time_stamp: row.get(3)?,
                    duration: row.get(4)?,
                },
                task_name: row.get(5)?,
            })
        })?;
        let mut events = vec![];
        for event in event_iter {
            let event = event.unwrap();
            events.push(event)
        }
        Ok(events)
    }
    /// get all the events
    pub fn get_events(&self) -> Result<Vec<EventWithTaskName>> {
        let stmt = &mut self.connection.prepare(
            "SELECT 
                event.id AS event_id,
                event.task_id AS event_task_id,
                event.notes AS event_notes,
                event.time_stamp AS event_time_stamp,
                event.duration AS event_duration,
                task.name AS task_name
                FROM 
                    event
                LEFT JOIN 
                    task ON event.task_id = task.id
            ",
        )?;
        let event_iter = stmt.query_map([], |row| {
            Ok(EventWithTaskName {
                event: Event {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    notes: row.get(2)?,
                    time_stamp: row.get(3)?,
                    duration: row.get(4)?,
                },
                task_name: row.get(5)?,
            })
        })?;
        let mut events = vec![];
        for event in event_iter {
            let event = event.unwrap();
            events.push(event)
        }
        Ok(events)
    }
    pub fn get_time_spent_by_task(&self, task_id: i32) -> Result<i32> {
        let stmt = &mut self.connection.prepare(
            "SELECT 
            SUM(duration)
            FROM
            event
            WHERE event.task_id= $1",
        )?;
        let mut rows = stmt.query([task_id])?;
        let mut time_spent = 0;
        while let Some(row) = rows.next()? {
            time_spent = row.get(0)?;
        }
        Ok(time_spent)
    }
}
