use clap::Parser;
use std::sync::{Arc, Mutex};
use std::{
    io::{self, Read, Write},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
struct Task {
    id: i32,
    name: String,
    details: Option<String>,
}

fn print_elapsed_time(elapsed_time: Duration) {
    let total_seconds = elapsed_time.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let milliseconds = elapsed_time.subsec_millis();

    print!("\r{:02}:{:02}.{:03}", minutes, seconds, milliseconds);
    io::stdout().flush().unwrap();
}

fn user_interupt() -> Option<u8> {
    let mut buffer = [0; 1];
    let stdin = io::stdin();

    match stdin.lock().read_exact(&mut buffer) {
        Ok(_) => Some(buffer[0]),
        Err(_) => None,
    }
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

fn main() {
    let cli = Cli::parse();
    println!("task: {:?}", cli.task);

    let should_terminate = Arc::new(Mutex::new(AtomicBool::new(false)));
    let should_terminate_thread = should_terminate.clone();

    let start_time = Instant::now();
    let handle = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1));
        print_elapsed_time(start_time.elapsed());
        io::stdout().flush().unwrap();
        if should_terminate_thread
            .lock()
            .unwrap()
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            break;
        }
    });
    // Simulate user interaction in the main thread
    // For example, wait for the user to press Enter to terminate the loop
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Set the should_terminate flag to true to signal the loop to terminate
    should_terminate
        .lock()
        .unwrap()
        .store(true, Ordering::Relaxed);

    // Wait for the loop thread to finish
    handle.join().expect("The loop thread panicked");

    println!("\nTask complete! Elapsed time: {:?}", start_time.elapsed())
}
