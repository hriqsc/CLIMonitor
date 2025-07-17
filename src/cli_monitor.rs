use ratatui::{
    crossterm::event::{self, Event, KeyCode}, 
    layout::{Constraint, Direction, Layout}, 
    style::{Modifier, Style}, 
    widgets::{Block, Borders, Cell, Row, Table, Wrap}, 
    Frame
};
use reqwest::Client;
use std::{collections::HashSet, error::Error};
use ratatui::style::Color;
use crate::{api_service::{self, Entry}, config, modal};

pub struct CliMonitor {
    pub selected: i32,
    pub on_modal: bool,
    pub current_modal : Modal,
    pub error: MonitorError,
    pub is_on_error: bool,
    pub is_adding_selected: bool,
    pub item_hash_set: HashSet<String>,
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
            selected: 0 , 
            on_modal: false, 
            current_modal: Modal::None, 
            error: MonitorError::None,
            is_on_error: false,
            is_adding_selected: false,
            item_hash_set: HashSet::new(),
        }
    }

    pub fn set_modal(&mut self, modal: Modal) {
        self.on_modal = true;
        self.current_modal = modal
    }
    
}

pub fn render(monitor : &CliMonitor,entries: &Vec<Entry>, f: &mut Frame) -> Result<(), Box<dyn Error>> {
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

    let rows = entries.iter().enumerate().map(|(i, row)| {
        let row_strs = {
            let mut row_strs = vec![];
            row_strs.push(row.user_name.clone());
            row_strs.push(row.machine_name.clone());
            row_strs.push(row.function.clone());
            row_strs.push(row.environment.clone());
            row_strs.push(row.time_up.clone());
            row_strs.push(row.thread_type.clone());
            row_strs
        };
        let cells = row_strs.iter().map(|col| Cell::from(col.clone()));
        
        let mut styled_row = Row::new(cells);
    
        if i as i32 == monitor.selected {
            styled_row = styled_row.style(Style::default().bg(Color::Gray).fg(Color::Black).add_modifier(Modifier::BOLD));
        }else if monitor.item_hash_set.contains(&row.id.clone()){
            styled_row = styled_row.style(Style::default().bg(Color::LightRed).fg(Color::Black).add_modifier(Modifier::BOLD));
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
        .borders(Borders::ALL)
    )
        
        .column_spacing(1);

    f.render_widget(table, chunks[0]);

    let footer = Block::default()
        .title("comandos")
        .border_style(Style::default().fg(Color::Yellow))
        .borders(Borders::ALL);
    f.render_widget(
        ratatui::widgets::Paragraph::new("Sair <q>  Desconectar <d>  Mensagem <m>  Mais detalhes <M>  Atualizar <a> Des/Seleciona <e> Limpa seleção <E> seleciona varios <tab>")
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
                                monitor.on_modal = false;
                                return Err(e);
                            }
                        };
                    }
                    Modal::None => {}
                }
                
                update(&token, &client, *page,  entries).await;
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
                let entrie_selected = &entries[monitor.selected as usize];
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Down => {
                        monitor.selected = (monitor.selected + 1) % entries.len() as i32;
                        
                        
                        if monitor.is_adding_selected{
                            let contains = monitor.item_hash_set.contains(&entrie_selected.id.clone());
                            if contains {
                                monitor.item_hash_set.remove(&entrie_selected.id.clone());
                            }else{
                                monitor.item_hash_set.insert(entrie_selected.id.clone());
                            }
                        }
                    }
                    KeyCode::Up => {
                        if monitor.selected > 0 {
                            monitor.selected -= 1;
                        } else {
                            monitor.selected = entries.len() as i32 - 1;
                        }

                        if monitor.is_adding_selected{
                            let contains = monitor.item_hash_set.contains(&entrie_selected.id.clone());
                            if contains {
                                monitor.item_hash_set.remove(&entrie_selected.id.clone());
                            }else{
                                monitor.item_hash_set.insert(entrie_selected.id.clone());
                            }
                        }
                    }
                    KeyCode::Right => *page += 1,
                    KeyCode::Left => if *page > 0 { *page -= 1 },
                    KeyCode::Char('a') => {
                        update(&token, &client, *page,  entries).await;
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
                    KeyCode::Tab => {
                        if monitor.is_adding_selected{
                            monitor.item_hash_set.insert(entrie_selected.id.clone());
                        }else{
                            monitor.item_hash_set.clear();
                        }
                        
                        monitor.is_adding_selected = !monitor.is_adding_selected;
                    }
                    
                    KeyCode::Char('e') =>{
                        let contains = monitor.item_hash_set.contains(&entrie_selected.id.clone());
                        if contains {
                            monitor.item_hash_set.remove(&entrie_selected.id.clone());
                        }else{
                            monitor.item_hash_set.insert(entrie_selected.id.clone());
                        }
                    }
                    KeyCode::Char('E') =>{
                        monitor.item_hash_set.clear();
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
) {
    *entries = api_service::get_entries(&token, &client, page, 10).await;
}
