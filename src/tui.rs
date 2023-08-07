use crate::types::types::Task;
use std::{io, thread, time::Duration};

use clap::Error;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
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

fn ui<B: Backend>(f: &mut Frame<B>, tasks: Vec<Task>) {
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
    let mut items = Vec::new();
    for task in tasks {
        items.push(ListItem::new(task.name))
    }
    let task_list = List::new(items.clone()).block(
        Block::default()
            .title("Tasks")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    let mut app = App::new(items);
    f.render_stateful_widget(task_list, chunks[1], &mut app.items.state);
    let block = Block::default()
        .title("Task Details")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(block, chunks[2]);
}

pub fn run_app(store: &Store) -> Result<(), Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tasks = store.get_tasks().unwrap();
    terminal.draw(|f| {
        ui(f, tasks);
    })?;

    // Start a thread to discard any input events. Without handling events, the
    // stdin buffer will fill up, and be read into the shell when the program exits.
    thread::spawn(|| loop {
        event::read();
    });

    thread::sleep(Duration::from_millis(5000));

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
