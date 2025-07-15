use ratatui::{
    crossterm::event::{self, Event, KeyCode}, 
    layout::{Constraint, Direction, Layout}, 
    style::{Modifier, Style}, 
    widgets::{Block, Borders, Cell, Row, Table, Wrap}, 
    Frame
};
use reqwest::Client;
use std::error::Error;
use ratatui::style::Color;
use crate::{api_service::{self, Entry}, config, modal};

pub struct CliMonitor {
    pub rows: Vec<Vec<String>>,
    pub selected: i32,
    pub on_modal: bool,
    pub current_modal : Modal,
    pub error: MonitorError,
    pub is_on_error: bool,
}

pub enum MonitorError{
    None,
    SendMsgError(String)
}

pub enum Modal{
    Delete,
    Info,
    SendMsg,
    None
}

impl CliMonitor {
    pub fn new() -> Self {
        Self { 
            rows: vec![], 
            selected: 0 , 
            on_modal: false, 
            current_modal: Modal::None, 
            error: MonitorError::None,
            is_on_error: false
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn add_entry(&mut self, entry: &Entry) {
        self.add_row(vec![
            entry.user_name.clone(),
            entry.machine_name.clone(),
            entry.function.clone(),
            entry.environment.clone(),
            entry.time_up.clone(),
            entry.thread_type.clone(),
        ]);
    }
    pub fn add_entry_page(&mut self, entries: &Vec<Entry>) {
        for entry in entries {
            self.add_entry(entry)
        }
    }
    pub fn clean(&mut self) {
        self.rows = vec![];
    }
    pub fn set_modal(&mut self, modal: Modal) {
        self.on_modal = true;
        self.current_modal = modal
    }
    
}

pub fn render(monitor : &CliMonitor, f: &mut Frame) -> Result<(), Box<dyn Error>> {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(size);

    let header_cells = ["usuario", "computador", "programa", "ambiente", "tempo de conexão", "tipo de conexão"]
        .iter()
        .map(|h| 
                Cell::from(*h)
                        .style(Style::default().add_modifier(Modifier::BOLD))
        );
                    
    let header = Row::new(header_cells).style(Style::default().fg(Color::White).bg(Color::Black));

    let rows = monitor.rows.iter().enumerate().map(|(i, row)| {
        let cells = row.iter().map(|col| Cell::from(col.clone()));
        
        let mut styled_row = Row::new(cells);
    
        if i as i32 == monitor.selected {
            styled_row = styled_row.style(Style::default().bg(Color::Gray).fg(Color::Black).add_modifier(Modifier::BOLD));
        }
    
        styled_row
    });

    let table = Table::new(
            rows,
            [
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(18),
                Constraint::Percentage(18),
            ],
        )
        .header(header)
        .block(Block::default().title("CLI Monitor")
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL))
        
        .column_spacing(1);

    f.render_widget(table, chunks[0]);

    let footer = Block::default()
        .title("comandos")
        .border_style(Style::default().fg(Color::Yellow))
        .borders(Borders::ALL);
    f.render_widget(
        ratatui::widgets::Paragraph::new("Sair <q>  Desconectar <d>  Mensagem <m>  Mais detalhes <M>  Atualizar <a>")
            .block(footer)
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
            )
            .wrap(Wrap { trim: true }),
        chunks[1],
    );

    Ok(())
}



pub async fn user_key_input(
    monitor : &mut CliMonitor, 
    entries: &mut Vec<Entry>, 
    page: &mut i32, 
    token: &str, 
    client: &Client, 
    config: &config::Config, 
    input_buffer: &mut String,
) -> Result<bool, MonitorError> {
    match event::read(){
        Ok(Event::Key(key)) => {
            if monitor.on_modal {
                let entry = &entries[monitor.selected as usize];
        
                match monitor.current_modal {
                    Modal::Delete => {
                        monitor.on_modal = modal::confirm_del_modal(&key, &entry.id, &token, &client, &config).await;
                    }
                    Modal::Info => {
                        monitor.on_modal = modal::more_info_keys(&key).await;
                    }
                    Modal::SendMsg => {
                        match modal::message_keys(&key, input_buffer, &entry, token, &client, &config).await{
                            Ok(b) => monitor.on_modal = b,
                            Err(e) => {
                                monitor.on_modal = true;
                                return Err(e);
                            }
                        };
                    }
                    Modal::None => {}
                }
                
                update(&token, &client, *page,  entries,  monitor).await;
            }else if monitor.is_on_error {
                
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                        monitor.is_on_error = false;
                        monitor.on_modal = false;
                        monitor.error = MonitorError::None;
                    }
                    _ => {}
                    
                }
            
            } else {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
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
                    KeyCode::Right => *page += 1,
                    KeyCode::Left => if *page > 0 { *page -= 1 },
                    KeyCode::Char('a') => {
                        update(&token, &client, *page,  entries, monitor).await;
                    }
                    KeyCode::Char('d') => {
                        monitor.set_modal(Modal::Delete);
                    }
                    KeyCode::Char('m') => {
                        monitor.set_modal(Modal::SendMsg);
                    }
                    KeyCode::Char('M') => {
                        monitor.set_modal(Modal::Info);
                    }
                    _ => {}
                }
            }
        }
        Err(e) => panic!("Error: {}", e),
        _ => {}
    }
    Ok(false)
}


pub async fn update(
    token: &str,
    client: &Client,
    page: i32,
    entries: &mut Vec<Entry>,
    monitor: &mut CliMonitor,
) {
    *entries = api_service::get_entries(&token, &client, page, 10).await;
    monitor.clean();
    monitor.add_entry_page(&entries);
}
