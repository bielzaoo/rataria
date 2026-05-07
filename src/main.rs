mod auth;
mod db;
mod error;

use db::Database;
use error::Result;

fn main() -> Result<()> {
    let db_path = Database::default_path();
    println!("Banco em: {:?}", db_path);

    let password = "senha_teste_123";
    let _db = Database::open(&db_path, password)?;
    println!("Banco aberto e schema criado com sucesso!");

    Ok(())
}
