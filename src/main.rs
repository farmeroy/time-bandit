use clap::{Args, Parser, Subcommand};
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

use types::types::Task;

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
    /// Start a new task
    Start(StartArgs),
    /// List completed tasks
    List,
    Tui,
    Events,
}

#[derive(Args)]
struct StartArgs {
    #[arg(long, short)]
    task: String,
    #[arg(long, short)]
    details: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let store = store::Store::new("./new_db.db3")?;
    match &cli.command {
        Commands::Start(task) => {
            println!("task: {:?}", task.task);
            println!("details: {}", task.details.clone().unwrap_or_default());
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
            //might not need to make a task
            // let task = Task {
            //     id: 0,
            //     name: task.task.to_string(),
            //     details: task.details.clone().unwrap_or_default(),
            //     // time_stamp: now.to_string(),
            //     // duration: format_elapsed_time(start_time.elapsed()),
            // };
            // maybe break this into two calls to the store:
            // get the id of the task name OR create it
            // then create a new event with that id
            store.add_task_event(
                task.task.to_string(),
                task.details.clone().unwrap_or_default(),
                now.to_string(),
                format_elapsed_time(start_time.elapsed()).to_string(),
            )?;

            println!(
                "\rTask complete! Elapsed time: {:?}",
                format_elapsed_time(start_time.elapsed())
            );
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
                    task.name,
                    task.details,
                    task.events.unwrap()
                );
                println!("id:{}: {}", task.id, formatted_task);
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
