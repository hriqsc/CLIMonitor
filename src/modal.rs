
use std::borrow::Cow;

use ratatui::{
    crossterm::event::{self, KeyCode}, prelude::*, widgets::{Block, Borders, Paragraph, Wrap}
};
use reqwest::Client;

use crate::{api_service, cli_monitor::MonitorError, config::Config};




pub async fn confirm_del_modal(
    key : &event::KeyEvent, 
    entry_id : &str, 
    token: &str, 
    client: &Client,
    config : &Config
)-> bool{
    match key.code {
        KeyCode::Char('s') => {
            api_service::delete_connection(config,entry_id, &token, &client).await;
            return true;
        }
        KeyCode::Char('n') => {
            return true;
            
        }
        _ => {
            return false;
        }
    }
}

pub fn draw_confirm_del_modal(f: &mut Frame){
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


pub async fn more_info_keys(key : &event::KeyEvent)-> bool{
    
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
            return false;
        }
        _ => {
            return true;
        }
    }
}


pub fn draw_more_info_modal(f: &mut Frame, entry: &api_service::Entry) {
    let area = centered_rect(40, 30, f.area());
    let block = Block::default()
        .title("Informações")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Black));

    let label_width = 15;
    let value_width = 27;

    // Tuplas (label1, value1, label2, value2)
    let rows = vec![
        ("Usuário",         Cow::Borrowed(entry.user_name.trim()),      "Computador",       Cow::Borrowed(entry.machine_name.trim())),
        ("Thread ID",       Cow::Owned(entry.thread_id.to_string()),    "Servidor",         Cow::Borrowed(entry.server.trim())),
        ("Programa",        Cow::Borrowed(entry.function.trim()),       "Ambiente",         Cow::Borrowed(entry.environment.trim())),
        ("Data/Hora",       Cow::Borrowed(entry.date_time.trim()),      "Tempo de conexão", Cow::Borrowed(entry.time_up.trim())),
        ("Instruções",      Cow::Owned(entry.instructions.to_string()), "Instruções P/s",   Cow::Owned(entry.instructions_ps.to_string())),
        ("Comentários",     Cow::Borrowed(entry.comments.trim()),       "Memória",          Cow::Owned(entry.memory.to_string())),
        ("SID",             Cow::Borrowed(entry.s_id.trim()),           "Ctree",            Cow::Owned(entry.id_ctree.to_string())),
        ("Tipo de conexão", Cow::Borrowed(entry.thread_type.trim()),    "Tempo inativo",    Cow::Borrowed(entry.inactive_time.trim())),
    ];

    fn truncate(s: &str, max: usize) -> String {
        if s.chars().count() > max {
            let mut truncated = String::new();
            for (i, c) in s.chars().enumerate() {
                if i >= max - 1 {
                    break;
                }
                truncated.push(c);
            }
            truncated.push('…');
            truncated
        } else {
            s.to_string()
        }
    }

    let mut lines: Vec<Line> = Vec::with_capacity(rows.len());
    lines.push("\n".into());
    for (l1, v1, l2, v2) in rows {
        lines.push(Line::from(vec![
            Span::raw(format!("{:<label_width$}:", l1, label_width = label_width)),
            Span::raw(format!("{:>value_width$}", truncate(&v1, value_width), value_width = value_width)),
            Span::raw(" | "),
            Span::raw(format!("{:<label_width$}:", l2, label_width = label_width)),
            Span::raw(format!("{:>value_width$}", truncate(&v2, value_width), value_width = value_width)),
        ]));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}



pub async fn message_keys(
    key : &event::KeyEvent, 
    input_buffer : &mut String, 
    entry: &api_service::Entry, 
    token: &str, 
    client: &Client,
    config : &Config
) -> Result<bool, MonitorError> {
    
    match key.code {
        KeyCode::Char(c) => {
            input_buffer.push(c);
            Ok(true)
        },
        KeyCode::Backspace => { 
            input_buffer.pop(); 
            Ok(true)
        },
        KeyCode::Enter => {
            let resp = api_service::send_message(config,&entry.id, &input_buffer, token, &client).await;
            if resp.status() != 200{
                let error = match resp.text().await{
                    Ok(error) => error,
                    Err(e) => panic!("Error: {}", e),
                };
                return Err(MonitorError::SendMsgError(error));
            }
            input_buffer.clear();
            Ok(false)
        },
        KeyCode::Esc => {
            input_buffer.clear();
            Ok(true)
        },
        _ => Ok(false)
    }

}



pub fn draw_send_message_modal(
    f: &mut Frame,
    entry: &api_service::Entry,
    input_buffer: &str,
) {
    let area = centered_rect(60, 20, f.area());

    let block = Block::default()
        .title(format!("Mensagem para {}", entry.user_name))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Black));

    let mut text = input_buffer.to_string();
    text.push('_');

    let paragraph = Paragraph::new(Text::from(Line::from(text)))
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White).bg(Color::Black));

    f.render_widget(paragraph, area);
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



pub fn draw_error(f: &mut Frame,tittle: &str, message: &str) {

    let area = centered_rect(30, 10, f.area());
    let block = Block::default()
        .title(tittle)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Red));

    let paragraph = Paragraph::new(Text::from(message))
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White).bg(Color::Red));

    f.render_widget(paragraph, area);
}