
#[allow(dead_code)]
#[derive(Debug)]
pub enum ConfigError{
    Parsing(String),
    ReadLineError(String),
    ReadWithMaskError(String),
    WriteFileError(String),
    ReadNumberLineError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Parsing(s) => write!(f, "Erro ao tentar parsear {}", s),
            ConfigError::ReadLineError(s) => write!(f, "Erro ao tentar ler linha {}", s),
            ConfigError::ReadWithMaskError(s) => write!(f, "Erro ao tentar ler senha {}", s),
            ConfigError::WriteFileError(s) => write!(f, "Erro ao tentar escrever arquivo {}", s),
            ConfigError::ReadNumberLineError(s) => write!(f, "Erro ao tentar ler linha numerica {}", s),
        }
    }
}



pub enum APIError{
    ParsingError(String),
    RequestError(String),
    AsyncError(String),
    AuthFail(String),
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            APIError::ParsingError(s) => write!(f, "Erro ao tentar parsear para json {}", s),
            APIError::RequestError(s) => write!(f, "Erro ao tentar fazer request {}", s),
            APIError::AsyncError(s) => write!(f, "Erro no processo assíncrono {}", s),
            APIError::AuthFail(s) => write!(f, "Falha na Autenticação {}", s),
        }
    }
}


pub enum TerminalError{
    AuthError(String),
    ConfigError(String),
    DrawError(String),
}

impl std::fmt::Display for TerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TerminalError::AuthError(s) => write!(f, "Erro ao tentar autenticar {}", s),
            TerminalError::ConfigError(s) => write!(f, "Erro ao tentar carregar config {}", s),
            TerminalError::DrawError(s) => write!(f, "Erro ao tentar desenhar terminal {}", s),
        }
    }
}

impl From<std::io::Error> for TerminalError {
    fn from(e: std::io::Error) -> Self {
        TerminalError::DrawError(e.to_string())
    }
}


impl From<ConfigError> for TerminalError {
    fn from(e: ConfigError) -> Self {
        TerminalError::ConfigError(e.to_string())
    }
}

impl From<APIError> for TerminalError {
    fn from(e: APIError) -> Self {
        TerminalError::AuthError(e.to_string())
    }
}


impl From<reqwest::Error> for TerminalError {
    fn from(e: reqwest::Error) -> Self {
        TerminalError::AuthError(e.to_string())
    }
}