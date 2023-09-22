use crate::types::types::Task;
use std::{
    io,
    time::{Duration, Instant},
};

use clap::Error;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
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
}
impl<T> App<T> {
    fn new(items: Vec<T>) -> App<T> {
        App {
            items: StatefulList::with_item(items),
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App<ListItem>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());
    let block = Block::default()
        .title("Time Bandit")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(block, chunks[0]);
    let task_list = List::new(app.items.items.clone())
        .block(
            Block::default()
                .title("Tasks")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(task_list, chunks[1], &mut app.items.state);
    let block = Block::default()
        .title("Task Details")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(block, chunks[2]);
}

pub fn run_app(store: Store) -> Result<(), Error> {
    let tasks = store.get_tasks().unwrap();
    let mut items = Vec::new();
    for task in tasks {
        items.push(ListItem::new(task.name.clone()))
    }
    let mut app = App::new(items);
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
            ui(f, &mut app);
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
