use aes_gcm::{
    aead::{rand_core::RngCore, Aead},
    Aes256Gcm, Key, KeyInit, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;

use crate::error::AppError;

// Derives a 256-bit AES key from a user password and random salt using Argon2id
fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32], AppError> {
    let argon2 = Argon2::default(); // Use Argon2id with default parameters (secure by default)

    // Encode the salt as a base64-compatible SaltString
    let salt_string = SaltString::encode_b64(salt).map_err(|e| AppError::Hashing(e.to_string()))?;

    // Hash the password using Argon2id and the given salt
    let hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| AppError::Hashing(format!("Error hashing key: {}", e.to_string())))?
        .hash
        .ok_or("missing hash")
        .map_err(|e| AppError::Hashing(e.to_string()))?;

    // Extract the first 32 bytes to use as an AES-256 key
    let key = hash.as_bytes()[..32]
        .try_into()
        .map_err(|_| AppError::Internal("Could not parse first 32 bytes".to_string()))?;

    Ok(key)
}

// Encrypts a file with a user-given password and writes salt + nonce + ciphertext to output
fn encrypt_file_with_password(
    input_data_as_bytes: Vec<u8>,
    output_path: &str,
    password: &str,
) -> Result<(), AppError> {
    // Generate a 16-byte random salt (used in key derivation)
    let mut salt = [0u8; 16];
    OsRng
        .try_fill_bytes(&mut salt)
        .map_err(|_| AppError::Internal("Error in generating 128-bit random salt".to_string()))?;

    // Generate a 12-byte nonce (required for AES-GCM)
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Derive a 32-byte key from the password + salt
    let key_bytes = derive_key_from_password(password, &salt)?;
    let key = Key::from_slice(&key_bytes); // Wrap key bytes for AES-GCM usage

    // Create AES-256-GCM cipher instance
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes); // Wrap nonce bytes

    // Encrypt the data using AES-GCM (authenticated encryption)
    let ciphertext = cipher
        .encrypt(nonce, input_data_as_bytes.as_ref())
        .map_err(|e| AppError::Internal(format!("Error in encrypting file: {}", e)))?;

    // Write the salt, nonce, and encrypted data to output file
    // let mut file = File::create(output_path)?;
    // file.write_all(&salt)?; // First 16 bytes: salt
    // file.write_all(&nonce_bytes)?; // Next 12 bytes: nonce
    // file.write_all(&ciphertext)?; // Remaining: encrypted content

    println!("{:?} {:?} {:?}", salt, nonce_bytes, ciphertext);

    println!("🔐 Encrypted file saved to: {}", output_path);
    Ok(())
}

pub fn main() {
    let input_file_content = fs::read_to_string("./hashing.rs").unwrap();

    let user_password = "12345";

    println!("{}", input_file_content);
}
