//! Live fullscreen popup (ratatui). Subscribes to the plugin SSE stream and
//! redraws on every change. `q`/`Esc`/`Ctrl-C` quits.

use std::io::{self, Stdout};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::client::Client;
use crate::model::Snapshot;

enum Msg {
    Snap(Snapshot),
    Disconnected,
}

pub fn run(port: u16) -> Result<(), String> {
    // Background thread: keep the SSE stream open, reconnecting on drop.
    let (tx, rx) = mpsc::channel::<Msg>();
    let stream_tx = tx.clone();
    thread::spawn(move || {
        let client = Client::new(port);
        loop {
            let tx2 = stream_tx.clone();
            let res = client.stream(|snap| tx2.send(Msg::Snap(snap)).is_ok());
            // Stream ended or failed to open: tell the UI, then retry shortly.
            if stream_tx.send(Msg::Disconnected).is_err() {
                return; // UI gone
            }
            let _ = res;
            thread::sleep(Duration::from_secs(2));
        }
    });

    let mut term = setup_terminal().map_err(|e| e.to_string())?;
    let result = ui_loop(&mut term, &rx);
    restore_terminal(&mut term).ok();
    result
}

fn ui_loop(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    rx: &Receiver<Msg>,
) -> Result<(), String> {
    let mut snapshot: Option<Snapshot> = None;
    let mut connected = false;

    loop {
        // Drain pending messages.
        loop {
            match rx.try_recv() {
                Ok(Msg::Snap(s)) => {
                    snapshot = Some(s);
                    connected = true;
                }
                Ok(Msg::Disconnected) => connected = false,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
        }

        term.draw(|f| draw(f, snapshot.as_ref(), connected))
            .map_err(|e| e.to_string())?;

        // Poll input briefly so the UI stays responsive to stream pushes.
        if event::poll(Duration::from_millis(200)).map_err(|e| e.to_string())? {
            if let Event::Key(k) = event::read().map_err(|e| e.to_string())? {
                let quit = matches!(k.code, KeyCode::Char('q') | KeyCode::Esc)
                    || (k.code == KeyCode::Char('c')
                        && k.modifiers.contains(KeyModifiers::CONTROL));
                if quit {
                    return Ok(());
                }
            }
        }
    }
}

fn draw(f: &mut Frame, snap: Option<&Snapshot>, connected: bool) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let (headline, kind) = match snap {
        Some(s) if !s.is_empty() => (
            if s.summary.headline.is_empty() {
                format!("{} sessions", s.sessions.len())
            } else {
                s.summary.headline.clone()
            },
            s.summary.headline_kind.clone(),
        ),
        Some(_) => ("No active sessions".to_string(), "idle".to_string()),
        None => ("Connecting…".to_string(), "idle".to_string()),
    };
    let header_style = match kind.as_str() {
        "permission" | "question" => Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        "busy" => Style::default().fg(Color::White),
        _ => Style::default().fg(Color::Gray),
    };
    let header = Paragraph::new(Line::from(Span::styled(headline, header_style)))
        .block(Block::default().borders(Borders::ALL).title(" SessionBar "));
    f.render_widget(header, chunks[0]);

    // Session list
    let items: Vec<ListItem> = match snap {
        Some(s) if !s.is_empty() => s
            .sessions
            .iter()
            .map(|e| {
                let glyph_style = match e.status.as_str() {
                    "waiting" => Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                    "busy" => Style::default().fg(Color::White),
                    "done" => Style::default().fg(Color::Green),
                    _ => Style::default().fg(Color::DarkGray),
                };
                let line = Line::from(vec![
                    Span::styled(format!("{} ", e.glyph()), glyph_style),
                    Span::raw(e.title.clone()),
                    Span::styled(
                        format!("  {}  ", e.detail),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("({})", e.age_label),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect(),
        Some(_) => vec![ListItem::new(Line::from(Span::styled(
            "  (no active sessions)",
            Style::default().fg(Color::DarkGray),
        )))],
        None => vec![],
    };
    let list = List::new(items).block(Block::default().borders(Borders::ALL));
    f.render_widget(list, chunks[1]);

    // Footer
    let status = if connected {
        "● live"
    } else {
        "○ reconnecting…"
    };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            status,
            Style::default().fg(if connected {
                Color::Green
            } else {
                Color::Yellow
            }),
        ),
        Span::styled("   q/Esc quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(term: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()
}
