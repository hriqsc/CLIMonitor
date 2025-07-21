use crossterm::{cursor, event::{read, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}, ExecutableCommand};
use serde::{Deserialize, Serialize};
use std::{fs, io::{stdout, Write}};
use std::io::stdin;

use crate::errors::ConfigError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub login: String,
    pub password: String,
    pub enviorment: String,
    pub refresh_interval_in_secs: u64,
    pub ip: String,
    pub porta: String,
    pub request_timeout_in_secs: u64,
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

pub fn load_config() -> Result<Config, ConfigError>{
    let path = "./config.toml";

    if fs::metadata(path).is_err() {
        println!("config.toml não encontrado");
        
        let config = match create_new_config(){
            Ok(config) => config,
            Err(e) => return Err(e),
        };
        let toml_str = match toml::to_string(&config){
            Ok(toml_str) => toml_str,
            Err(e) => return Err(ConfigError::Parsing(e.to_string())),
        };

        fs::write(path, toml_str).map_err(|e| ConfigError::WriteFileError(e.to_string()))?;
        
    }
    let toml_str : String = match fs::read_to_string(path) {
        Ok(toml_str) => toml_str,
        Err(e) => return Err(ConfigError::Parsing(e.to_string())),
    };

    let config : Config = match toml::from_str(&toml_str) {
        Ok(config) => config,
        Err(e) => return Err(ConfigError::Parsing(e.to_string())),
    };
    return Ok(config)


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
pub fn create_new_config() -> Result<Config, ConfigError>{

    fn read_t_line(prompt: &str) -> Result<String, ConfigError> {
        println!("{}", prompt);
        let mut input = String::new();
        match stdin().read_line(&mut input){
            Ok(_) => {}
            Err(e) => return Err(ConfigError::ReadLineError(e.to_string())),
        };
        println!();
        Ok(input)
    }

    fn read_t_numbers(prompt : &str) -> Result<u64, ConfigError> {
        println!("{}", prompt);
        let input : u64 = match read_only_numbers(){
            Ok(input) => input,
            Err(e) => return Err(e),
        };
        println!();
        Ok(input)
    }


    let login = read_t_line("Digite o login: ")?;

    println!("Digite a senha: ");
    let password = match read_with_mask('*') {
        Ok(password) => password,
        Err(e) => return Err(e),  
    };
    println!("");


    let enviorment = read_t_line("Digite o ambiente: ")?;

    
    let refresh_interval_in_secs: u64 = read_t_numbers("Digite o intervalo de atualizacao em segundos: ")?;

    let ip = read_t_line("Digite o ip: ")?;
    let porta = read_t_numbers("Digite a porta: ")?.to_string();

    let config = Config {
        login: login.trim().to_string(),
        password: password.trim().to_string(),
        enviorment: enviorment.trim().to_string(),
        refresh_interval_in_secs: refresh_interval_in_secs,
        ip: ip.trim().to_string(),
        porta: porta.trim().to_string(),
        request_timeout_in_secs: 15
    };

    Ok(config)
}

/// Reads a string from stdin, echoing each character as `mask` instead of the actual character.
///
/// This is useful for reading passwords from the user without echoing the password to the console.
///
/// # Return
///
/// A `String` containing the input string.
fn read_with_mask(mask : char) -> Result<String, ConfigError> {
    let mut stdout = stdout();
    let mut password = String::new();

    enable_raw_mode().map_err(|e| ConfigError::ReadWithMaskError(e.to_string()))?;
    stdout.execute(cursor::MoveToColumn(0)).map_err(|e| ConfigError::ReadWithMaskError(e.to_string()))?;
    stdout.flush().map_err(|e| ConfigError::ReadWithMaskError(e.to_string()))?;

    loop {
        if let Event::Key(event) = read().map_err(|e| ConfigError::ReadWithMaskError(e.to_string()))? {
            match event.code {
                KeyCode::Enter => {
                    println!();
                    break;
                }
                KeyCode::Char(c) => {
                    password.push(c);
                    print!("{}", mask);
                    stdout.flush().map_err(|e| ConfigError::ReadWithMaskError(e.to_string()))?;
                }
                KeyCode::Backspace => {
                    if password.pop().is_some() {
                        print!("\x08 \x08"); // backspace visual
                        stdout.flush().map_err(|e| ConfigError::ReadWithMaskError(e.to_string()))?;
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().map_err(|e| ConfigError::ReadWithMaskError(e.to_string()))?;
    Ok(password)
}


/// Reads a number from the user by processing key events.
///
/// This function enables raw mode and starts reading key events. It will only accept numeric
/// characters and backspace. When Enter is pressed, it will parse the input string as a number
/// and return it. If there is an error parsing the number, or if there is an error reading the
/// key event, it will return an error.
///
/// # Return
///
/// A `Result` containing the input number, or a `ConfigError` if there is an error.
fn read_only_numbers() -> Result<u64, ConfigError> {
    let mut stdout = stdout();
    let mut numbers = String::new();

    enable_raw_mode().map_err(|e| ConfigError::ReadNumberLineError(e.to_string()))?;
    stdout.execute(cursor::MoveToColumn(0)).map_err(|e| ConfigError::ReadNumberLineError(e.to_string()))?;
    stdout.flush().map_err(|e| ConfigError::ReadNumberLineError(e.to_string()))?;

    loop {
        if let Event::Key(event) = read().map_err(|e| ConfigError::ReadNumberLineError(e.to_string()))? {
            match event.code {
                KeyCode::Enter => {
                    println!();
                    break;
                }
                KeyCode::Backspace => {
                    if numbers.pop().is_some() {
                        print!("\x08 \x08"); // backspace visual
                        stdout.flush().map_err(|e| ConfigError::ReadNumberLineError(e.to_string()))?;
                    }
                }
                KeyCode::Char(c) => {
                    if c.is_numeric() {
                        numbers.push(c);
                        print!("{}", c);
                        stdout.flush().map_err(|e| ConfigError::ReadNumberLineError(e.to_string()))?;
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().map_err(|e| ConfigError::ReadNumberLineError(e.to_string()))?;

    let numbers_result : u64 = numbers.parse().map_err(|e : std::num::ParseIntError| ConfigError::ReadNumberLineError(e.to_string()))?;

    Ok(numbers_result)
}

