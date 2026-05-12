use crate::error::{RatariaError, Result};
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use zeroize::Zeroizing;

/// Deriva uma chave a partir da senha usando Argon2id
/// Retorna a chave como String protegida pelo Zeroizing
#[allow(dead_code)]
pub fn derive_key(password: &str) -> Result<Zeroizing<String>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| RatariaError::CryptoError(e.to_string()))?;

    // Zeroizing garante que a string é zerada da memória ao sair do escopo
    Ok(Zeroizing::new(hash.to_string()))
}

/// Verifica se a senha bate com o hash armazenado
#[allow(dead_code)]
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash).map_err(|e| RatariaError::CryptoError(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
