use crate::{format_elapsed_time, types::types::Task};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use clap::Error;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::Style,
    symbols::{self, block},
    text::Line,
    widgets::{
        Block, BorderType, Borders, Dataset, List, ListItem, ListState, Paragraph, Row, Table,
    },
    Frame, Terminal,
};

use crate::store::Store;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}
impl<T> StatefulList<T> {
    fn with_item(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<T> {
    items: StatefulList<T>,
    tasks: Vec<Task>,
    events: Option<Vec<Event>>, // this is not currently used
    timer: Timer,
}
impl<T> App<T> {
    fn new(items: Vec<T>, tasks: Vec<Task>) -> App<T> {
        App {
            items: StatefulList::with_item(items),
            tasks,
            events: None,
            timer: Timer::new(),
        }
    }
}

#[derive(Debug)]
struct Timer {
    is_on: Arc<AtomicBool>,
    start_time: Option<DateTime<Local>>,
    elapsed_time: Arc<Mutex<Option<u64>>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Timer {
    fn new() -> Timer {
        Timer {
            is_on: Arc::new(AtomicBool::new(false)),
            start_time: None,
            elapsed_time: Arc::new(Mutex::new(None)),
            thread_handle: None,
        }
    }
    fn start(&mut self) {
        self.start_time = Some(Local::now());
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
    }
    fn stop(&mut self) {
        self.is_on.store(false, Ordering::Relaxed);
        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle.join().unwrap();
        }
    }
    fn clear(&mut self) {
        self.start_time = None;
        self.elapsed_time.lock().unwrap().take();
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App<ListItem>, store: &Store) {
    let selected_task_index = app.items.state.selected().unwrap_or_default();
    let selected_task_name = app.tasks[selected_task_index].name.clone();
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(45),
                Constraint::Percentage(45),
            ]
            .as_ref(),
        )
        .split(f.size());
    let middle_rectangles = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Max(30), Constraint::Percentage(60)].as_ref())
        .split(vertical_layout[1]);

    // timer view
    let elapsed_time = {
        let elapsed_time_guard = app.timer.elapsed_time.lock().unwrap();
        *elapsed_time_guard
    };
    let timer = Paragraph::new(format!(
        "Task: {} Start Time: {} Elapsed Time: {}",
        &selected_task_name,
        app.timer.start_time.unwrap_or_default(),
        elapsed_time.unwrap_or_default()
    ))
    .block(
        Block::default()
            .title("Time Bandit")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(timer, vertical_layout[0]);

    let task_list = List::new(app.items.items.clone())
        .block(
            Block::default()
                .title("Tasks")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(task_list, middle_rectangles[0], &mut app.items.state);

    let events = store.get_events_by_task(&selected_task_name);
    let events = events.unwrap();
    // let datasets = vec![Dataset::default()
    //     .name("events")
    //     .marker(symbols::Marker::Braille)
    //     .style(Style::default().fg(ratatui::style::Color::Yellow))
    //     .graph_type(ratatui::widgets::GraphType::Line)
    //     .data()];

    let rows = events.clone().into_iter().map(|event| {
        Row::new(vec![
            event
                .event
                .time_stamp
                .parse::<DateTime<Local>>()
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            format_elapsed_time(event.event.duration as u64),
            event.event.notes.unwrap_or("---".to_string()),
        ])
    });

    let events_table = Table::new(rows)
        .header(Row::new(vec!["Time Stamp", "Duration", "Notes"]))
        .block(
            Block::default()
                .title(format!("Details for '{}' Events", selected_task_name))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .widths(&[
            Constraint::Max(30),
            Constraint::Max(15),
            Constraint::Min(30),
        ]);
    f.render_widget(events_table, vertical_layout[2]);
}

pub fn run_app(store: Store) -> Result<(), Error> {
    let tasks = store.get_tasks().unwrap();
    let mut items = Vec::new();
    for task in &tasks {
        items.push(ListItem::new(task.name.clone()))
    }
    let mut app = App::new(items, tasks);
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    // Start a thread to discard any input events. Without handling events, the
    // stdin buffer will fill up, and be read into the shell when the program exits.
    loop {
        terminal.draw(|f| {
            ui(f, &mut app, &store);
        })?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => app.items.next(),
                        KeyCode::Char('k') => app.items.previous(),
                        KeyCode::Char('s') => app.timer.start(),
                        KeyCode::Char('d') => app.timer.stop(),
                        _ => {}
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
