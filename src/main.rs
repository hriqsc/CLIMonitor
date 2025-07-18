use api_service::Entry;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::{DefaultTerminal, Frame};
use reqwest::Client;
use std::time::Duration;
use tokio::{sync::mpsc, time::interval};

mod cli_monitor;
mod api_service;
mod modal;
mod config;

#[tokio::main]
async fn main() {
    
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);

    match result.await{
        Ok(_) => {}
        Err(e) => {
            terminal.clear().unwrap();
            println!("Error: {}", e);
        },
    }
    ratatui::restore();
}



/// Main entry point of the application. This function will start the application in a full screen terminal.
///
/// # Errors
///
/// If there is an error drawing the terminal, this function will return an error.
///
async fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut monitor = cli_monitor::CliMonitor::new();
    let mut page = 0;
    let client = Client::new();
    let config = config::load_config();
    let mut token: String = api_service::get_token(&config,&client,).await;
    let mut entries: Vec<Entry> = api_service::get_entries(&token, &client, page, 10).await;
    let (tx, mut rx) = mpsc::unbounded_channel::<TimerEvent>();
    let mut input_buffer = String::new();

    enable_raw_mode()?;
    create_timer(&tx, TimerEvent::Refresh ,Duration::from_secs(config.refresh_interval_in_secs));
    create_timer(&tx, TimerEvent::Every30Min ,Duration::from_secs(30 * 3600));
    terminal.clear()?;

    loop {
        terminal.draw(|f| draw(f, &mut monitor, &entries, &mut input_buffer))?;
        
        let has_exited = cli_monitor::user_key_input(
            &mut monitor, 
            &mut entries, 
            &mut page, 
            &token, 
            &client, 
            &config, 
            &mut input_buffer
        ).await;
        
        match has_exited {
            Ok(b) => {
                if b {
                    break;
                }
            },
            Err(e) => {
                monitor.error = e;
                monitor.is_on_error = true;
            }
        };
        
        
        if let Ok(Some(_)) = rx.try_recv().map(Some) {
            cli_monitor::update(&token, &client, page, &mut entries).await;
        }

        if let Ok(Some(event)) = rx.try_recv().map(Some) {
            match event {
                TimerEvent::Refresh => {
                    cli_monitor::update(&token, &client, page, &mut entries).await;
                },
                TimerEvent::Every30Min => {
                    token = api_service::get_token(&config,&client,).await;
                }
            }
        }
        
    }

    disable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}



/// Renders the terminal interface, including modals if they are active.
///
/// This function draws the main interface and handles any errors that occur during rendering.
/// It also checks if a modal is active and draws the appropriate modal based on the current state
/// of the `CliMonitor`.
///
/// # Arguments
///
/// * `f` - The frame to render the interface.
/// * `monitor` - The CLI monitor state, which tracks the current modal and error state.
/// * `entries` - A vector of entries representing the data to be displayed.
/// * `input_buffer` - A buffer containing the user's input for the message modal.
fn draw(f: &mut Frame, monitor: &mut cli_monitor::CliMonitor, entries: &Vec<Entry>, input_buffer: &mut String) {
    if let Err(e) = cli_monitor::render(&monitor, entries,f) {
        println!("Error: {}", e);
    }

    match &monitor.error {
        cli_monitor::MonitorError::None => {}
        cli_monitor::MonitorError::SendMsgError(msg) => {
            modal::draw_error(f, "Erro ao enviar mensagem", &msg);
        }
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


#[derive(Copy, Clone)]
enum TimerEvent {
    Refresh,
    Every30Min,
}
fn create_timer(tx : &mpsc::UnboundedSender<TimerEvent>,event : TimerEvent, duration : Duration) {
    tokio::spawn({
        let tx = tx.clone();
        async move {
            let mut ticker = interval(duration);
            loop {
                ticker.tick().await;
                let _ = tx.send(event);
            }
        }
    });
}