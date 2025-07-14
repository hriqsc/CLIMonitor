use ratatui::{
    layout::{Constraint, Direction, Layout}, style::{Modifier, Style}, widgets::{Block, Borders, Cell, Row, Table, Wrap}, Frame
};
use std::error::Error;
use ratatui::style::Color;
use crate::api_service::Entry;

pub struct CliMonitor {
    pub rows: Vec<Vec<String>>,
    pub selected: i32,
    pub on_modal: bool,
    pub current_modal : Modal,
}

pub enum Modal{
    Delete,
    Info,
    SendMsg,
    None
}

impl CliMonitor {
    pub fn new() -> Self {
        Self { rows: vec![], selected: 0 , on_modal: false, current_modal: Modal::None}
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


    pub fn render(&self, f: &mut Frame) -> Result<(), Box<dyn Error>> {
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

        let rows = self.rows.iter().enumerate().map(|(i, row)| {
            let cells = row.iter().map(|col| Cell::from(col.clone()));
            
            let mut styled_row = Row::new(cells);
        
            if i as i32 == self.selected {
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
}