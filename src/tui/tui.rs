use crate::tui::{stateful_list::StatefulList, timer::Timer};
use crate::{format_elapsed_time, types::types::Task};
use std::{
    io,
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
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};

use crate::store::Store;

struct App<T> {
    items: StatefulList<T>,
    tasks: Vec<Task>,
    active_task: Option<Task>,
    timer: Timer,
}
impl<T> App<T> {
    fn new(items: Vec<T>, tasks: Vec<Task>) -> App<T> {
        App {
            items: StatefulList::with_item(items),
            tasks,
            active_task: None,
            timer: Timer::new(),
        }
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
    let active_task_name = match &app.active_task {
        Some(task) => task.name.clone(),
        None => "".to_string(),
    };
    let timer = Paragraph::new(format!(
        "Task: {} Start Time: {} Elapsed Time: {}",
        active_task_name,
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
                .title(format!("Events for '{}'", selected_task_name))
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
                        KeyCode::Char('s') => {
                            app.timer.start();
                            app.active_task =
                                Some(app.tasks[app.items.state.selected().unwrap()].clone())
                        }
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
