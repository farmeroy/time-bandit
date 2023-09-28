use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct Timer {
    pub is_on: Arc<AtomicBool>,
    pub start_time: Option<DateTime<Local>>,
    pub elapsed_time: Arc<Mutex<Option<u64>>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            is_on: Arc::new(AtomicBool::new(false)),
            start_time: None,
            elapsed_time: Arc::new(Mutex::new(None)),
            thread_handle: None,
        }
    }
    pub fn start(&mut self) {
        if self.is_on.load(Ordering::Relaxed) {
            return;
        };
        self.is_on.store(true, Ordering::Relaxed);
        let is_on = Arc::clone(&self.is_on);
        let elapsed_time = Arc::clone(&self.elapsed_time);
        let start_time = Instant::now();
        let thread_handle = thread::spawn(move || loop {
            let mut elapsed_time = elapsed_time.lock().unwrap();
            *elapsed_time = Some(start_time.elapsed().as_secs());
            if !is_on.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
        });
        self.thread_handle = Some(thread_handle);
        self.start_time = Some(Local::now());
    }
    pub fn stop(&mut self) {
        self.is_on.store(false, Ordering::Relaxed);
        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle.join().unwrap();
        }
    }
    pub fn clear(&mut self) {
        self.start_time = None;
        self.elapsed_time.lock().unwrap().take();
    }
}
