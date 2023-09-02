#![allow(dead_code)]

use clap::{Parser, Subcommand};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use std::{
    io::{self, Read, Write},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;

use crate::types::types::EventWithTaskName;

mod store;
mod types;

fn format_elapsed_time(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}h:{:02}m:{:02}s", hours, minutes, seconds)
}

#[derive(Parser)]
#[command(author = "Raffaele Cataldo")]
#[command(name = "tb")]
#[command(version = "1.0")]
#[command(about = "Keep track of time wasted on tasks", long_about =None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage your various tasks
    #[command(subcommand)]
    Task(TaskAction),
    /// View the time stolen from you
    Events(EventsArgs),
}

#[derive(Parser)]
struct TaskStartArgs {
    #[arg(index = 1)]
    /// Name the task you want to work on
    name: String,
    #[arg(long, short)]
    /// Add task details
    details: Option<String>,
}

#[derive(Parser)]
struct EventsArgs {
    /// The name of the task
    #[arg(index = 1)]
    name: Option<String>,
}

#[derive(Subcommand)]
enum TaskAction {
    /// Start the task
    Start(TaskStartArgs),
    /// List all tasks
    List,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let store = store::Store::new("./new_db.db3")?;
    match &cli.command {
        Commands::Task(action) => {
            match &action {
                TaskAction::List => {
                    let task_iter = store.get_tasks().unwrap();
                    println!("Task Name | Total Time Spent");
                    for task in task_iter {
                        let task = task;
                        let time_spent = store.get_time_spent_by_task(task.id).unwrap();
                        println!(
                            "{} | {}",
                            task.name,
                            format_elapsed_time(time_spent.try_into()?)
                        );
                    }
                }
                TaskAction::Start(args) => {
                    println!("task: {:?}", args.name);
                    println!("details: {}", args.details.clone().unwrap_or_default());
                    // capture the moment the task was begun
                    let now = Utc::now();
                    let should_terminate = Arc::new(Mutex::new(AtomicBool::new(false)));
                    let should_terminate_thread = should_terminate.clone();

                    let start_time = Instant::now();
                    let handle = thread::spawn(move || loop {
                        thread::sleep(Duration::from_millis(1));
                        print!("\r{}", format_elapsed_time(start_time.elapsed().as_secs()));
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
                    store.add_task_event(
                        args.name.to_string(),
                        args.details.clone().unwrap_or_default(),
                        now.to_string(),
                        start_time.elapsed().as_secs().try_into()?,
                    )?;

                    println!(
                        "\rTask complete! Elapsed time: {:?}",
                        format_elapsed_time(start_time.elapsed().as_secs())
                    );
                }
            }
        }
        Commands::Events(args) => {
            let task_name = args.name.clone();
            let events_iter: Vec<EventWithTaskName>;
            println!("Task Name | Event ID | Time Stamp | Duration | Notes");
            if let Some(task) = task_name {
                events_iter = store.get_events_by_task(task)?;
            } else {
                events_iter = store.get_events().unwrap();
            }
            let mut time_spent = 0;
            let mut total_events = 0;
            for event in events_iter {
                let event = event;
                time_spent += event.event.duration;
                total_events += 1;
                println!(
                    "{} | {} | {} | {} | {}",
                    event.task_name,
                    event.event.id,
                    event.event.time_stamp,
                    format_elapsed_time(event.event.duration.try_into()?),
                    event.event.notes.unwrap_or_default()
                );
            }
            println!(
                "Total time spent: {}\nTotal Events: {}",
                format_elapsed_time(time_spent as u64),
                total_events
            );
        }
    }

    Ok(())
}
