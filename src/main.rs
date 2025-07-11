use api_service::Entry;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use reqwest::Client;
use std::time::Duration;
use tokio::{sync::mpsc, time::interval};

mod cli_monitor;
mod api_service;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);

    ratatui::restore();
    result.await
}

async fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> std::io::Result<()> {
    let mut monitor = cli_monitor::CliMonitor::new();
    let mut page = 0;
    let client = Client::new();

    let login_request = api_service::AuthRequest {
        login: "".to_string(),
        password: "".to_string(),
        env: "".to_string(),
    };

    let token: String = api_service::get_token(&client, &login_request).await;
    let mut entries: Vec<Entry> = api_service::get_entries(&token, &client, page, 10).await;
    monitor.add_entry_page(&entries);

    enable_raw_mode()?;
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();

    tokio::spawn({
        let tx = tx.clone();
        async move {
            let mut ticker = interval(Duration::from_secs(5));
            loop {
                ticker.tick().await;
                let _ = tx.send(());
            }
        }
    });

    loop {
        terminal.draw(|f| draw(f, &mut monitor))?;

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if monitor.confirming_disconnect {
                    match key.code {
                        KeyCode::Char('s') => {
                            let entry = &entries[monitor.selected as usize];
                            api_service::delete_connection(&entry.id, &token, &client).await;
                            update(&token, &client, page, &mut entries, &mut monitor).await;
                            monitor.confirming_disconnect = false;
                        }
                        KeyCode::Char('n') => {
                            monitor.confirming_disconnect = false;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => {
                            monitor.selected = (monitor.selected + 1) % monitor.rows.len() as i32;
                        }
                        KeyCode::Up => {
                            if monitor.selected > 0 {
                                monitor.selected -= 1;
                            } else {
                                monitor.selected = monitor.rows.len() as i32 - 1;
                            }
                        }
                        KeyCode::Right => page += 1,
                        KeyCode::Left => if page > 0 { page -= 1 },
                        KeyCode::Char('a') => {
                            update(&token, &client, page, &mut entries, &mut monitor).await;
                        }
                        KeyCode::Char('d') => {
                            monitor.confirming_disconnect = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Ok(Some(_)) = rx.try_recv().map(Some) {
            update(&token, &client, page, &mut entries, &mut monitor).await;
        }
    }

    disable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}

async fn update(
    token: &str,
    client: &Client,
    page: i32,
    entries: &mut Vec<Entry>,
    monitor: &mut cli_monitor::CliMonitor,
) {
    *entries = api_service::get_entries(&token, &client, page, 10).await;
    monitor.clean();
    monitor.add_entry_page(&entries);
}

fn draw(f: &mut Frame, monitor: &mut cli_monitor::CliMonitor) {
    if let Err(e) = monitor.render(f) {
        println!("Error: {}", e);
    }

    if monitor.confirming_disconnect {
        let area = centered_rect(30, 10, f.area());
        let block = Block::default()
            .title("Confirmar desconexão?")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White).bg(Color::Red));

        let paragraph = Paragraph::new("Tem certeza que deseja desconectar? (s/n)")
            .style(Style::default().fg(Color::White).bg(Color::Red))
            .block(block)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 

// No seu cli_monitor.rs adicione este campo no struct CliMonitor:
// pub confirming_disconnect: bool
// e inicialize como false no método `new()`.
