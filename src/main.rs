use clap::Parser;
use std::sync::{Arc, Mutex};
use std::{
    io::{self, Read, Write},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use rusqlite::{params, Connection, Result};

#[derive(Debug)]
struct Task {
    id: i32,
    name: String,
    details: String,
}

fn format_elapsed_time(elapsed_time: Duration) -> String {
    let total_seconds = elapsed_time.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let milliseconds = elapsed_time.subsec_millis();

    format!("{:02}m:{:02}s.{:03}ms", minutes, seconds, milliseconds)
}

#[derive(Parser)]
#[command(author = "Raffaele Cataldo")]
#[command(name = "TimeBandit")]
#[command(version = "1.0")]
#[command(about = "Keep track of time wasted on tasks", long_about =None)]
struct Cli {
    #[arg(long, short)]
    task: String,
    #[arg(long, short)]
    details: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("task: {:?}", &cli.task);
    println!("details: {}", &cli.details.clone().unwrap_or_default());

    let conn = Connection::open("./my_db.db3")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            details TEXT
    )",
        (),
    )?;

    let should_terminate = Arc::new(Mutex::new(AtomicBool::new(false)));
    let should_terminate_thread = should_terminate.clone();

    let start_time = Instant::now();
    let handle = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1));
        print!("\r{}", format_elapsed_time(start_time.elapsed()));
        io::stdout().flush().unwrap();
        if should_terminate_thread
            .lock()
            .unwrap()
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            break;
        }
    });
    // wait for the user to press Enter to terminate the loop
    let mut buffer = [0u8; 1];
    io::stdin().read(&mut buffer).expect("Failed to read line");

    // Set the should_terminate flag to true to signal the loop to terminate
    should_terminate
        .lock()
        .unwrap()
        .store(true, Ordering::Relaxed);

    // Wait for the loop thread to finish
    handle.join().expect("The loop thread panicked");

    println!(
        "\rTask complete! Elapsed time: {:?}",
        format_elapsed_time(start_time.elapsed())
    );

    let task = Task {
        id: 0,
        name: cli.task.to_string(),
        details: cli.details.unwrap_or_default(),
    };

    conn.execute(
        "INSERT INTO task (name, details) VALUES (?1, ?2)",
        (&task.name, &task.details),
    )?;

    let mut stmt = conn.prepare("SELECT id, name, details FROM task")?;
    let task_iter = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            name: row.get(1)?,
            details: row.get(2)?,
        })
    })?;

    for task in task_iter {
        println!("Found task {:?}", task.unwrap());
    }

    Ok(())
}
