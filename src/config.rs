use crossterm::{cursor, event::{read, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}, ExecutableCommand};
use serde::{Deserialize, Serialize};
use std::{fs, io::{stdout, Write}};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub login: String,
    pub password: String,
    pub enviorment: String,
    pub refresh_interval_in_secs: u64,
    pub ip: String,
    pub porta: String
}


/// Loads the configuration from a TOML file.
///
/// This function attempts to read and parse the configuration data from a `config.toml` file
/// located in the current directory. If the file does not exist, it creates a new configuration
/// by prompting the user for input. The function returns a `Config` struct with the loaded
/// configuration data.
///
/// # Panics
///
/// This function will panic if there is an error reading the file or parsing the TOML data.

pub fn load_config() -> Config{
    let path = "./config.toml";

    if fs::metadata(path).is_err() {
        println!("config.toml não encontrado");
        create_new_config();

    }

    let toml_str = match fs::read_to_string(path) {
        Ok(toml_str) => toml_str,
        Err(e) => panic!("Error: {}", e),   
    };

    let config: Config = match toml::from_str(&toml_str) {
        Ok(config) => config,
        Err(e) => panic!("Error: {}", e),
    };

    config
}


/// Cria um novo arquivo de configura o.
///
/// Essa função é utilizada quando o arquivo de configura o n o existe.
/// Ela pergunta ao usu rio para digitar as informa es da configura o e
/// as salva em um novo arquivo de configura o.
///
/// # Retorno
///
/// Retorna um objeto `Config` com as informa es da configura o.
pub fn create_new_config() -> Config{
    print!("Digite o login: ");
    let mut login = String::new();
    std::io::stdin().read_line(&mut login).unwrap();
    print!("Digite a senha: ");
    let password = read_with_mask('*');
    print!("Digite o ambiente: ");
    let mut enviorment = String::new();
    std::io::stdin().read_line(&mut enviorment).unwrap();
    print!("Digite o intervalo de atualizacao em segundos: ");
    let mut refresh_interval_in_secs = String::new();
    std::io::stdin().read_line(&mut refresh_interval_in_secs).unwrap();
    let mut ip = String::new();
    print!("Digite o ip: ");
    std::io::stdin().read_line(&mut ip).unwrap();
    let mut porta = String::new();
    print!("Digite a porta: ");
    std::io::stdin().read_line(&mut porta).unwrap();

    Config {
        login: login.trim().to_string(),
        password: password.trim().to_string(),
        enviorment: enviorment.trim().to_string(),
        refresh_interval_in_secs: refresh_interval_in_secs.trim().parse().unwrap(),
        ip: ip.trim().to_string(),
        porta: porta.trim().to_string()
    }
}

/// Reads a string from stdin, echoing each character as `mask` instead of the actual character.
///
/// This is useful for reading passwords from the user without echoing the password to the console.
///
/// # Return
///
/// A `String` containing the input string.
fn read_with_mask(mask : char) -> String {
    let mut stdout = stdout();
    let mut password = String::new();

    enable_raw_mode().unwrap();
    stdout.execute(cursor::MoveToColumn(0)).unwrap();
    stdout.flush().unwrap();

    loop {
        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Enter => {
                    println!();
                    break;
                }
                KeyCode::Char(c) => {
                    password.push(c);
                    print!("{}", mask);
                    stdout.flush().unwrap();
                }
                KeyCode::Backspace => {
                    if password.pop().is_some() {
                        print!("\x08 \x08"); // backspace visual
                        stdout.flush().unwrap();
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
    password
}