use sqlx::{sqlite::SqliteRow, Result, Row, SqlitePool};
use types::{Event, EventWithTaskName, Task, TaskWithEvents};
pub mod types;

#[derive(Debug, Clone)]
pub struct Store {
    connection: SqlitePool,
}

impl Store {
    pub async fn new(db_url: &str) -> Result<Self> {
        let conn = SqlitePool::connect(db_url).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            details TEXT
    )",
        )
        .execute(&conn)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS event (
                id INTEGER PRIMARY KEY,
                task_id INTEGER,
                notes TEXT,
                time_stamp STRING,
                duration INTEGER,
                FOREIGN KEY(task_id) REFERENCES task(id)
            )",
        )
        .execute(&conn)
        .await
        .unwrap();
        Ok(Store { connection: conn })
    }
    /// Add an event associated with a task,
    /// creating a new task if it doesn't exist
    pub async fn add_task_event(
        &self,
        name: String,
        details: String,
        now: String,
        duration: u64,
    ) -> Result<Event> {
        if let Some(task_id) = self.get_task_id_by_name(&name).await.unwrap() {
            match self.create_event(task_id, &details, &now, duration).await {
                Ok(event) => {
                    println!("{:?}", event);
                    Ok(event)
                }
                Err(e) => {
                    println!("{:?}", e);
                    Err(e)
                }
            }
        } else {
            let new_task = self.create_task(&name, &details).await.unwrap();
            println!("{:?}", new_task);
            match self
                .create_event(new_task.id, &details, &now, duration)
                .await
            {
                Ok(event) => {
                    println!("{:?}", event);
                    Ok(event)
                }
                Err(e) => {
                    println!("{:?}", e);
                    Err(e)
                }
            }
        }
    }
    /// Fetch all tasks
    pub async fn get_tasks(&self) -> Result<Vec<Task>> {
        let tasks = sqlx::query(
            "
            SELECT * FROM task
            ",
        )
        .map(|row: SqliteRow| Task {
            id: row.get("id"),
            name: row.get("name"),
            details: row.get("details"),
        })
        .fetch_all(&self.connection)
        .await
        .unwrap();

        Ok(tasks)
    }
    /// Fetch all tasks together with their associated events
    pub async fn get_tasks_with_events(&self) -> Result<Vec<TaskWithEvents>> {
        let tasks = self.get_tasks().await.unwrap();
        let mut tasks_with_events: Vec<TaskWithEvents> = Vec::new();

        while let Some(task) = tasks.iter().next().cloned() {
            let _ = match sqlx::query("SELECT * FROM event WHERE event.task_id = $1")
                .bind(task.id)
                .map(|row: SqliteRow| Event {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    notes: row.get("notes"),
                    time_stamp: row.get("time_stamp"),
                    duration: row.get("duration"),
                })
                .fetch_all(&self.connection)
                .await
            {
                Ok(events) => Ok(tasks_with_events.push(TaskWithEvents {
                    task,
                    events: Some(events),
                })),
                Err(e) => Err(e),
            };
        }
        Ok(tasks_with_events)
    }
    /// Get events according to task name
    pub async fn get_events_by_task(&self, task_name: String) -> Result<Vec<EventWithTaskName>> {
        match sqlx::query(
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
        )
        .bind(task_name)
        .map(|row: SqliteRow| EventWithTaskName {
            event: Event {
                id: row.get("event_id"),
                task_id: row.get("event_task_id"),
                notes: row.get("event_notes"),
                time_stamp: row.get("event_time_stamp"),
                duration: row.get("event_duration"),
            },
            task_name: row.get("task_name"),
        })
        .fetch_all(&self.connection)
        .await
        {
            Ok(events) => Ok(events),
            Err(e) => Err(e),
        }
    }
    /// get all the events
    pub async fn get_events(&self) -> Result<Vec<EventWithTaskName>> {
        match sqlx::query(
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
        )
        .map(|row: SqliteRow| EventWithTaskName {
            event: Event {
                id: row.get("event_id"),
                task_id: row.get("event_task_id"),
                notes: row.get("event_notes"),
                time_stamp: row.get("event_time_stamp"),
                duration: row.get("event_duration"),
            },
            task_name: row.get("task_name"),
        })
        .fetch_all(&self.connection)
        .await
        {
            Ok(events) => Ok(events),
            Err(e) => Err(e),
        }
    }
    pub async fn get_time_spent_by_task(&self, task_id: i32) -> Result<i32> {
        match sqlx::query(
            "SELECT 
            SUM(duration)
            FROM
            event
            WHERE event.task_id= $1",
        )
        .bind(task_id)
        .fetch_one(&self.connection)
        .await
        {
            Ok(duration) => Ok(duration.get(0)),
            Err(e) => Err(e),
        }
    }
    pub async fn get_task_id_by_name(&self, task_name: &str) -> Result<Option<i32>> {
        match sqlx::query("SELECT id FROM task WHERE name = ?")
            .bind(task_name)
            .map(|row: SqliteRow| row.get("id"))
            .fetch_optional(&self.connection)
            .await
        {
            Ok(id) => Ok(id),
            Err(e) => Err(e),
        }
    }

    pub async fn create_task(&self, task_name: &str, details: &str) -> Result<Task> {
        match sqlx::query(
            "INSERT INTO task (name, details) 
            VALUES (?1, ?2)
            RETURNING id, name, details",
        )
        .bind(task_name)
        .bind(details)
        .map(|row: SqliteRow| Task {
            id: row.get("id"),
            name: row.get("name"),
            details: Some(row.get("details")),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(task) => Ok(task),
            Err(e) => Err(e),
        }
    }

    pub async fn create_event(
        &self,
        task_id: i32,
        notes: &str,
        now: &str,
        duration: u64,
    ) -> Result<Event> {
        println!("{}", notes);
        match sqlx::query(
            "INSERT INTO event (task_id, notes, time_stamp, duration) 
            VALUES (?1, ?2, ?3, ?4) 
            RETURNING id, task_id, notes, time_stamp, duration",
        )
        .bind(task_id)
        .bind(notes)
        .bind(now)
        .bind(duration as i32)
        .map(|row: SqliteRow| Event {
            id: row.get("id"),
            task_id: row.get("task_id"),
            notes: Some(row.get("notes")),
            time_stamp: row.get("time_stamp"),
            duration: row.get("duration"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(event) => {
                println!("{:?}", event);
                Ok(event)
            }
            Err(e) => {
                println!("err");
                Err(e)
            }
        }
    }
}
