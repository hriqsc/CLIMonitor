use api_service::Entry;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::{DefaultTerminal, Frame};
use reqwest::Client;
use std::time::Duration;
use tokio::{sync::mpsc, time::interval};

mod cli_monitor;
mod api_service;
mod modal;
mod config;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    
    let terminal = ratatui::init();
    let result = run(terminal);

    ratatui::restore();
    result.await
}

async fn run(mut terminal: DefaultTerminal) -> std::io::Result<()> {
    let mut monitor = cli_monitor::CliMonitor::new();
    let mut page = 0;
    let client = Client::new();
    let config = config::load_config();

    let token: String = api_service::get_token(&config,&client,).await;
    let mut entries: Vec<Entry> = api_service::get_entries(&token, &client, page, 10).await;
    monitor.add_entry_page(&entries);

    enable_raw_mode()?;
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();

    let mut input_buffer = String::new();

    tokio::spawn({
        let tx = tx.clone();
        async move {
            let mut ticker = interval(Duration::from_secs(config.refresh_interval_in_secs));
            loop {
                ticker.tick().await;
                let _ = tx.send(());
            }
        }
    });

    loop {
        if !event::poll(Duration::from_millis(10))?{
            continue
        }
        
        
        if let Event::Key(key) = event::read()? {
            if monitor.on_modal {
                let entry = &entries[monitor.selected as usize];

                match monitor.current_modal {
                    cli_monitor::Modal::Delete => {
                        monitor.on_modal = modal::confirm_del_modal(&key, &entry.id, &token, &client, &config).await;
                    }
                    cli_monitor::Modal::Info => {
                        monitor.on_modal = modal::more_info_keys(&key).await;
                    }
                    cli_monitor::Modal::SendMsg => {
                        monitor.on_modal = modal::message_keys(&key, &mut input_buffer, &entry, &token, &client, &config).await;
                    }
                    cli_monitor::Modal::None => {}
                }
                
                update(&token, &client, page, &mut entries, &mut monitor).await;
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
                        monitor.set_modal(cli_monitor::Modal::Delete);
                    }
                    KeyCode::Char('m') => {
                        monitor.set_modal(cli_monitor::Modal::Info);
                    }
                    KeyCode::Char('M') => {
                        monitor.set_modal(cli_monitor::Modal::SendMsg);
                    }
                    _ => {}
                }
            }
        }
            
        
        
        if let Ok(Some(_)) = rx.try_recv().map(Some) {
            update(&token, &client, page, &mut entries, &mut monitor).await;
        }
        
        terminal.draw(|f| draw(f, &mut monitor, &entries, &mut input_buffer))?;
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

fn draw(f: &mut Frame, monitor: &mut cli_monitor::CliMonitor, entries: &Vec<Entry>, input_buffer: &mut String) {
    if let Err(e) = monitor.render(f) {
        println!("Error: {}", e);
    }
    
    if monitor.on_modal {
        match monitor.current_modal {
            cli_monitor::Modal::Delete =>{
                modal::draw_confirm_del_modal(f);
            }
            cli_monitor::Modal::Info => {
                
                let entry: &Entry = &entries[monitor.selected as usize];
                modal::draw_more_info_modal(f, entry);
            }
            cli_monitor::Modal::SendMsg => {
                let entry: &Entry = &entries[monitor.selected as usize];
                modal::draw_send_message_modal(f,&entry, input_buffer);
            }
            cli_monitor::Modal::None => {}
        }
    }
}
