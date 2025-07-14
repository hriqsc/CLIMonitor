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


pub fn load_config() -> Config{
    let path = "./config.toml";

    if fs::metadata(path).is_err() {
        println!("Config file not found");
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