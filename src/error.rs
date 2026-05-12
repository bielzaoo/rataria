use thiserror::Error;

#[derive(Error, Debug)]
pub enum RatariaError {
    #[error("Erro no banco de dados: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Senha incorreta ou banco corrompido")]
    #[allow(dead_code)]
    WrongPassword,

    #[error("Não encontrado: {0}")]
    NotFound(String),

    #[error("Erro de importação: {0}")]
    ImportError(String),

    #[error("Erro ao criar diretório de dados: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Erro de criptografia: {0}")]
    CryptoError(String),
}

pub type Result<T> = std::result::Result<T, RatariaError>;
