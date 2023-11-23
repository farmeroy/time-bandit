#![allow(dead_code)]

use clap::{Parser, Subcommand};
use tokio;
#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};
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

use dirs::home_dir;

use chrono::{DateTime, Local};

use store::types::EventWithTaskName;

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
    /// View events associated with tasks
    Events(EventsArgs),
}

#[derive(Parser)]
struct TaskStartArgs {
    #[arg(index = 1)]
    /// Name the task you want to work on
    name: String,
    #[arg(long, short)]
    /// Add task details. The first time you start a task,
    /// the task details will be associated with the task itself.
    /// After that, task details will be notes for individual events.
    details: Option<String>,
}

#[derive(Parser)]
struct EventsArgs {
    #[arg(index = 1)]
    /// Enter the name of task who's events you would like to view
    name: Option<String>,
}

#[derive(Subcommand)]
enum TaskAction {
    /// Start the task
    Start(TaskStartArgs),
    /// List all tasks
    List,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let home_dir = home_dir().unwrap();
    let store =
        store::Store::new(format!("{}/.time_bandit.db3", home_dir.to_string_lossy()).as_str())
            .await
            .unwrap();
    match &cli.command {
        Commands::Task(action) => {
            match &action {
                TaskAction::List => {
                    let task_iter = store.get_tasks().await.unwrap();
                    let mut table = Table::new();
                    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
                    table.set_titles(row!["Task Name", "Total Time Spent"]);
                    for task in task_iter {
                        let task = task;
                        let time_spent = store.get_time_spent_by_task(task.id).await.unwrap();

                        table.add_row(row![task.name, format_elapsed_time(time_spent.try_into()?)]);
                    }
                    table.printstd();
                }
                TaskAction::Start(args) => {
                    println!("Task: {}", args.name);
                    println!("Details: {}", args.details.clone().unwrap_or_default());
                    let now = Local::now();
                    let should_terminate = Arc::new(Mutex::new(AtomicBool::new(false)));
                    let should_terminate_thread = should_terminate.clone();

                    let start_time = Instant::now();
                    println!("Press Enter to stop");
                    let handle = thread::spawn(move || {
                        // wait for the user to press Enter to terminate the loop
                        let mut buffer = [0u8; 1];
                        io::stdin().read(&mut buffer).expect("Failed to read line");

                        // Set the should_terminate flag to true to signal the loop to terminate
                        should_terminate
                            .lock()
                            .unwrap()
                            .store(true, Ordering::Release);
                    });
                    loop {
                        print!("\r{}", format_elapsed_time(start_time.elapsed().as_secs()));

                        io::stdout().flush().unwrap();
                        if should_terminate_thread
                            .lock()
                            .unwrap()
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            break;
                        }
                        thread::sleep(Duration::from_millis(250));
                    }
                    // Wait for the loop thread to finish
                    handle.join().expect("The loop thread panicked");
                    let event = match store
                        .add_task_event(
                            args.name.to_string(),
                            args.details.clone().unwrap_or_default(),
                            now.to_string(),
                            start_time.elapsed().as_secs().try_into()?,
                        )
                        .await
                    {
                        Ok(event) => Ok(event),
                        Err(e) => Err(e),
                    };
                    println!("{:?}", event);
                    // here we sleep the thread to ensure the event is written to sqlite
                    // A better solution may be to wrap this entire process in its own
                    // async function?
                    thread::sleep(Duration::from_millis(250));
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
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            table.set_titles(row!["ID", "Task Name", "Time Stamp", "Duration", "Notes"]);
            if let Some(task) = task_name {
                events_iter = store.get_events_with_task_name(task).await.unwrap();
            } else {
                events_iter = store.get_events().await.unwrap();
            }
            let mut time_spent = 0;
            let mut total_events = 0;
            for event in events_iter {
                let event = event;
                time_spent += event.event.duration;
                total_events += 1;

                table.add_row(row![
                    event.event.id,
                    event.task_name,
                    event
                        .event
                        .time_stamp
                        .parse::<DateTime<Local>>()?
                        .format("%Y-%m-%d %H:%M"),
                    format_elapsed_time(event.event.duration.try_into()?),
                    event.event.notes.unwrap_or_default(),
                ]);
            }
            table.printstd();
            println!(
                "Total time spent: {}\nTotal Events: {}",
                format_elapsed_time(time_spent as u64),
                total_events
            );
        }
    }

    Ok(())
}
