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

mod store;
mod tui;
mod types;

fn format_elapsed_time(elapsed_time: Duration) -> String {
    let total_seconds = elapsed_time.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let milliseconds = elapsed_time.subsec_millis();

    format!(
        "{:02}h:{:02}m:{:02}s.{:03}ms",
        hours, minutes, seconds, milliseconds
    )
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
    /// List completed tasks
    List,
    Tui,
    Events,
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
struct TaskEventsArgs {
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
    /// List events associated with a task
    Events(TaskEventsArgs),
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let store = store::Store::new("./new_db.db3")?;
    match &cli.command {
        Commands::Task(action) => {
            match &action {
                TaskAction::Events(args) => {
                    let task_name = args.name.clone();
                    if let Some(task) = task_name {
                        let events_iter = store.get_events_by_task(task).unwrap();
                        println!("Task Name | Event ID | Time Stamp | Duration");
                        for event in events_iter {
                            let event = event;
                            println!(
                                "{} | {} | {} | {}",
                                event.task_name,
                                event.event.id,
                                event.event.time_stamp,
                                event.event.duration
                            );
                        }
                    } else {
                        let events_iter = store.get_events().unwrap();
                        println!("Task Name | Event ID | Time Stamp | Duration");
                        for event in events_iter {
                            let event = event;
                            println!(
                                "{} | {} | {} | {}",
                                event.task_name,
                                event.event.id,
                                event.event.time_stamp,
                                event.event.duration
                            );
                        }
                    }
                }
                TaskAction::List => {
                    let task_iter = store.get_tasks().unwrap();
                    for task in task_iter {
                        let task = task;
                        println!("Task: {}", task.name);
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
                    store.add_task_event(
                        args.name.to_string(),
                        args.details.clone().unwrap_or_default(),
                        now.to_string(),
                        format_elapsed_time(start_time.elapsed()).to_string(),
                    )?;

                    println!(
                        "\rTask complete! Elapsed time: {:?}",
                        format_elapsed_time(start_time.elapsed())
                    );
                }
            }
        }
        Commands::List => {
            let task_iter = store.get_tasks_with_events().unwrap();
            for task in task_iter {
                let task = task;
                let formatted_task = format!(
                    "TASK NAME: {}, 
                    \ndetails: {:?},
                    \nevents: {:?}
                    ",
                    task.task.name,
                    task.task.details,
                    task.events.unwrap()
                );
                println!("id:{}: {}", task.task.id, formatted_task);
            }
        }
        Commands::Tui => tui::run_app(store)?,
        Commands::Events => {
            let event_iter = store.get_events().unwrap();
            for event in event_iter {
                let event = event;
                let formatted_event = format!(
                    "timestamp: {}, task: {}, duration: {}",
                    event.event.time_stamp, event.task_name, event.event.duration
                );
                println!("{}", formatted_event);
            }
        }
    }

    Ok(())
}
