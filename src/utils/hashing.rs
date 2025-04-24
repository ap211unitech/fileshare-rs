use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use crate::error::AppError;

pub fn hash_secret(secret: &str) -> Result<String, AppError> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = argon2
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| AppError::Hashing(format!("Error in hashing token: {}", e)))?
        .to_string();

    Ok(hashed_password)
}

pub fn verify_secret(hashed_secret: &str, given_value: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash =
        PasswordHash::new(&hashed_secret).map_err(|e| AppError::Hashing(e.to_string()))?;

    Ok(argon2
        .verify_password(given_value.as_bytes(), &parsed_hash)
        .is_ok())
}
