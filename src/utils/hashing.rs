use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use crate::error::AppError;

/// Hashes a given secret (e.g., a password or token) using the Argon2 algorithm.
///
/// # Arguments
/// * `secret` - A string slice representing the secret to be hashed.
///
/// # Returns
/// * `Ok(String)` containing the hashed representation of the secret.
/// * `Err(AppError)` if the hashing process fails.
pub fn hash_secret(secret: &str) -> Result<String, AppError> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = argon2
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| AppError::Hashing(format!("Error in hashing token: {}", e)))?
        .to_string();

    Ok(hashed_password)
}

/// Verifies whether a given value matches the previously hashed secret.
///
/// # Arguments
/// * `hashed_secret` - The stored hashed secret (as a string).
/// * `given_value` - The raw input value to verify against the hash.
///
/// # Returns
/// * `Ok(true)` if the input matches the hashed secret.
/// * `Ok(false)` if it does not match.
/// * `Err(AppError)` if verification or hash parsing fails.
pub fn verify_secret(hashed_secret: &str, given_value: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash =
        PasswordHash::new(&hashed_secret).map_err(|e| AppError::Hashing(e.to_string()))?;

    Ok(argon2
        .verify_password(given_value.as_bytes(), &parsed_hash)
        .is_ok())
}
