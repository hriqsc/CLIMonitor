
use std::borrow::Cow;

use ratatui::{
    crossterm::event::{self, KeyCode}, prelude::*, widgets::{Block, Borders, Paragraph, Wrap}, DefaultTerminal
};
use reqwest::Client;

use crate::{api_service, cli_monitor::MonitorError, config::Config, errors::TerminalError};




/// Handles key events for the delete confirmation modal.
///
/// This function processes key inputs to confirm or cancel the deletion
/// of connections. If the 's' key is pressed, the function triggers the
/// deletion of connections using the provided entry IDs and returns false,
/// indicating the modal should close. If the 'n' key is pressed, it cancels
/// the deletion and also returns false to close the modal. Any other key
/// keeps the modal open.
///
/// # Arguments
///
/// * `key` - The key event to process.
/// * `entries_id` - A vector of entry IDs to be deleted if confirmed.
/// * `token` - The authorization token for the API request.
/// * `client` - The HTTP client used to make the API request.
/// * `config` - The configuration used for the API request.
///
/// # Returns
///
/// Returns `false` if the modal should close, and `true` if it should remain open.
pub async fn confirm_del_modal(
    key : &event::KeyEvent, 
    entries_id : &Vec<String>, 
    token: &str, 
    client: &Client,
    config : &Config
)-> bool{
    match key.code {
        KeyCode::Char('s') => {
            api_service::delete_connections(config,entries_id, &token, &client).await;
            return false;
        }
        KeyCode::Char('n') => {
            return false;
            
        }
        _ => {
            return true;
        }
    }
}

/// Renderiza um modal de confirma o para desconectar conex es do Protheus.
/// 
/// # Argumentos
/// 
/// * `f` - frame que ser  renderizado.
/// 
/// # Retorno
/// 
/// Nenhum retorno.
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


/// Processa teclas pressionadas enquanto o modal de mais informa es est  aberto.
/// 
/// # Argumentos
/// 
/// * `key` - tecla pressionada.
/// 
/// # Retorno
/// 
/// Retorna true se o modal de mais informa es deve permanecer aberto. Caso contr rio, retorna false.
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


/// Renderiza um modal com informa es sobre a conex o selecionada.
/// 
/// # Argumentos
/// 
/// * `f` - frame que ser  renderizado.
/// * `entry` - estrutura que cont m as informa es sobre a conex o.
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



/// Handles key events for the message sending modal.
///
/// This function processes key inputs to control the message sending modal.
/// Characters are appended to the input buffer, and backspace removes the last character.
/// The Enter key sends the message using the provided entry IDs and returns a result indicating success or failure.
/// The Esc key clears the input buffer and closes the modal.
///
/// # Arguments
///
/// * `key` - The key event to process.
/// * `input_buffer` - The buffer containing the message to be sent.
/// * `entries` - A vector of entry IDs to send the message to.
/// * `token` - The authorization token for the API request.
/// * `client` - The HTTP client used to make the API request.
/// * `config` - The configuration used for the API request.
///
/// # Returns
///
/// Returns `Ok(true)` if the modal should remain open, `Ok(false)` if it should close, 
/// and `Err(MonitorError::SendMsgError)` if there is an error sending the message.
pub async fn message_keys(
    key : &event::KeyEvent, 
    input_buffer : &mut String, 
    entries: &Vec<String>, 
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
            let resp = api_service::send_messages(config,&entries, &input_buffer, token, &client).await;
            
            input_buffer.clear();

            match resp {
                Ok(resp_msg) => {
                    let msg = match resp_msg.message{
                        Some(msg) => msg,
                        None => "".to_string(),
                    };
        
        
                    if msg == ""{
                        Ok(false)
                    }else{
                        Err(MonitorError::SendMsgError(msg))
                    }

                }
                Err(e) => return Err(MonitorError::SendMsgError(e.to_string())),
            }
            
        },
        KeyCode::Esc => {
            input_buffer.clear();
            Ok(false)
        },
        _ => Ok(false)
    }

}



/// Renderiza um modal para enviar uma mensagem para o usu rio.
///
/// # Argumentos
///
/// * `f` - frame que ser  renderizado.
/// * `entry` - estrutura que cont m as informa es sobre a conex o.
/// * `input_buffer` - buffer de entrada com o texto da mensagem que o usu rio est  digitando.
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
    


/// Calculate the position of a centered rect with the given percentage of the area.
///
/// The given rect is split into three parts, and the middle part is split again
/// into three parts. The middle part of the middle part is the return value.
///
/// # Arguments
///
/// * `percent_x`: Percentage of the x axis for the width of the rect.
/// * `percent_y`: Percentage of the y axis for the height of the rect.
/// * `r`: The area to split.
///
/// # Returns
///
/// The position and size of the centered rect.
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



/// Renderiza um modal de erro com uma mensagem para o usu rio.
///
/// # Argumentos
///
/// * `f` - frame que ser  renderizado.
/// * `tittle` - t tulo do modal.
/// * `message` - mensagem a ser exibida no modal.
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



pub fn draw_loading(terminal: &mut DefaultTerminal) -> Result<(), TerminalError>{
    terminal.draw(|f| {
        let size = f.area();

        let block = Block::default()
            .title("Aguarde")
            .borders(Borders::ALL);
        
        let paragraph = Paragraph::new(Line::from(vec![
            Span::styled("Carregando...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]))
        .alignment(Alignment::Center)
        .block(block);

        f.render_widget(paragraph, size);
    })?;
    Ok(())
}