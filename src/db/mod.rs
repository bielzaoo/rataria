mod migrations;
use crate::error::Result;
use migrations::SCHEMA_V1;
use rusqlite::Connection;
use std::path::PathBuf;

pub mod models;
pub mod queries;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Abre (ou cria) o banco criptografado com a senha fornecida
    pub fn open(db_path: &PathBuf, password: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(&format!("PRAGMA key = '{}';", password))?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            PRAGMA synchronous = NORMAL;
        ",
        )?;
        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    /// Abre um banco em memória para testes
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA key = 'test_key';")?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
        ",
        )?;
        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA_V1)?;
        Ok(())
    }

    /// Retorna o caminho padrão do banco seguindo XDG
    pub fn default_path() -> PathBuf {
        let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("rataria");
        std::fs::create_dir_all(&dir).ok();
        dir.join("rataria.db")
    }
}
