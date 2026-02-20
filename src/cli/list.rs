use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{
    io::{self, Stdout},
    time::Duration,
};

use crate::{
    error::AutoPilotError,
    fs::get_autopilot_path,
    job::set::remove_job,
    status::{JobStatusStruct, get::get_status_log, set::set_status_initial},
};

// ─────────────────────────────────────────────────────────────
// App State
// ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
enum AppMode {
    Browse,
    ViewingDetails,
    ConfirmDelete,
}

struct JobListApp {
    jobs: Vec<JobStatusStruct>,
    list_state: ListState,
    mode: AppMode,
    exit: bool,
    status_message: Option<String>,
}

impl JobListApp {
    fn new() -> Result<Self, AutoPilotError> {
        set_status_initial().map_err(|e| AutoPilotError::Unknown(e))?;
        let status_log = get_status_log();
        let jobs = status_log.statuses;

        let mut list_state = ListState::default();
        if !jobs.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            jobs,
            list_state,
            mode: AppMode::Browse,
            exit: false,
            status_message: None,
        })
    }

    fn selected_job(&self) -> Option<&JobStatusStruct> {
        self.list_state.selected().and_then(|i| self.jobs.get(i))
    }

    fn delete_selected(&mut self) -> Result<(), AutoPilotError> {
        if let Some(job) = self.selected_job() {
            remove_job(Some(job.id.clone()), None)?;
            self.status_message = Some(format!("✓ Deleted job {}", job.id));
            // Refresh job list
            set_status_initial().map_err(|e| AutoPilotError::Unknown(e))?;
            self.jobs = get_status_log().statuses;
            // Adjust selection
            if self.jobs.is_empty() {
                self.list_state.select(None);
            } else {
                let max = self.jobs.len().saturating_sub(1);
                let current = self.list_state.selected().unwrap_or(0);
                self.list_state.select(Some(current.min(max)));
            }
        }
        Ok(())
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), AutoPilotError> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            // Poll for events with timeout for responsive UI
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    self.handle_key(key.code);
                }
            }

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::Browse => match key {
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.jobs.is_empty() {
                        let i = self.list_state.selected().unwrap_or(0);
                        self.list_state
                            .select(Some((i + 1).min(self.jobs.len() - 1)));
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.jobs.is_empty() {
                        let i = self.list_state.selected().unwrap_or(0);
                        self.list_state.select(Some(i.saturating_sub(1)));
                    }
                }
                KeyCode::Enter => {
                    if self.selected_job().is_some() {
                        self.mode = AppMode::ViewingDetails;
                    }
                }
                KeyCode::Delete | KeyCode::Char('d') | KeyCode::Backspace => {
                    if self.selected_job().is_some() {
                        self.mode = AppMode::ConfirmDelete;
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                _ => {}
            },

            AppMode::ViewingDetails => match key {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('b') => {
                    self.mode = AppMode::Browse;
                }
                KeyCode::Delete | KeyCode::Char('d') | KeyCode::Backspace => {
                    if self.selected_job().is_some() {
                        self.mode = AppMode::ConfirmDelete;
                    }
                }
                _ => {}
            },

            AppMode::ConfirmDelete => match key {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    if let Err(e) = self.delete_selected() {
                        self.status_message = Some(format!("✗ Error: {}", e));
                    }
                    self.mode = AppMode::Browse;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Delete => {
                    self.mode = AppMode::Browse;
                }
                _ => {}
            },
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.area());

        match self.mode {
            AppMode::Browse => self.render_browse(f, chunks[0]),
            AppMode::ViewingDetails => self.render_details(f, chunks[0]),
            AppMode::ConfirmDelete => self.render_confirm_delete(f, chunks[0]),
        }

        self.render_status_bar(f, chunks[1]);
    }

    fn render_browse(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .jobs
            .iter()
            .map(|job| {
                let content = Line::from(vec![
                    Span::styled(&job.id, Style::default().fg(Color::Yellow)),
                    Span::raw(" - "),
                    Span::styled(&job.name, Style::default().fg(Color::Green)),
                ]);
                ListItem::new(content)
            })
            .collect();

        let title = format!("Jobs [root: {}]", get_autopilot_path());
        let hint = if self.jobs.is_empty() {
            "No jobs found — press ESC to exit"
        } else {
            "↑↓/j/k navigate • Enter: details • Del/d: delete • ESC/q: quit"
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▸ ");

        f.render_stateful_widget(list, area, &mut self.list_state.clone());

        // Render hint below list
        let hint_para = Paragraph::new(Span::styled(hint, Style::default().fg(Color::DarkGray)))
            .style(Style::default().fg(Color::DarkGray));
        let hint_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        f.render_widget(hint_para, hint_area);
    }

    fn render_details(&self, f: &mut Frame, area: Rect) {
        let Some(job) = self.selected_job() else {
            return;
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&job.id, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&job.name, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:?}", job.status)),
            ]),
            Line::raw(""),
            Line::from(Span::styled(
                "Press ESC/Enter to go back • Del/d to delete",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let block = Block::default()
            .title("Job Details")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
    }

    fn render_confirm_delete(&self, f: &mut Frame, area: Rect) {
        let Some(job) = self.selected_job() else {
            return;
        };

        let lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled("Delete job ", Style::default().fg(Color::White)),
                Span::styled(
                    &job.id,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("?", Style::default().fg(Color::White)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&job.name, Style::default().fg(Color::Green)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled(
                    "[Y]es",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" / "),
                Span::styled(
                    "[N]o",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" • ESC to cancel"),
            ]),
        ];

        let block = Block::default()
            .title("⚠️  Confirm Deletion")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightRed));

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let msg = match &self.status_message {
            Some(m) => m.clone(),
            None => format!("Mode: {:?} • Jobs: {}", self.mode, self.jobs.len()),
        };
        let style = if self.status_message.is_some() {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let bar = Paragraph::new(format!(" {} ", msg)).style(style);
        f.render_widget(bar, area);
    }
}

// ─────────────────────────────────────────────────────────────
// Public Entry Point
// ─────────────────────────────────────────────────────────────
pub fn list() {
    match run_tui() {
        Ok(_) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn run_tui() -> Result<(), AutoPilotError> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = {
        let mut app = JobListApp::new()?;
        app.run(&mut terminal)
    };

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
