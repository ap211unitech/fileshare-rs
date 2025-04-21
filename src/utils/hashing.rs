use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

pub fn hash_password(password: &str) -> Result<String, String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();

    Ok(hashed_password)
}

pub fn verify_password(hashed_password: &str, password: &str) -> Result<bool, String> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&hashed_password).map_err(|e| e.to_string())?;

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
