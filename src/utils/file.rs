use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use aes_gcm::{
    aead::{rand_core::RngCore, Aead},
    Aes256Gcm, Key, KeyInit, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use chrono::Utc;

use crate::error::AppError;

/// Derives a 256-bit (32-byte) AES key from a user-provided password and a given salt using Argon2id.
///
/// # Arguments
/// * `password` - The user's password from which the key will be derived.
/// * `salt` - A 16-byte random salt for key derivation.
///
/// # Returns
/// * `Ok([u8; 32])` containing the derived key.
/// * `Err(AppError)` if the hash fails or cannot extract key material.
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32], AppError> {
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

/// Encrypts data using AES-256-GCM with a password-derived key. Output format: salt + nonce + ciphertext.
///
/// # Arguments
/// * `input_data_as_bytes` - The plaintext data to encrypt.
/// * `password` - The password used to derive the encryption key.
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the encrypted data.
/// * `Err(AppError)` if encryption or key derivation fails.
pub fn encrypt_file_with_password(
    input_data_as_bytes: Vec<u8>,
    password: &str,
) -> Result<Vec<u8>, AppError> {
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
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes); // Wrap key bytes for AES-GCM usage

    // Create AES-256-GCM cipher instance
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes); // Wrap nonce bytes

    // Encrypt the data using AES-GCM (authenticated encryption)
    let ciphertext = cipher
        .encrypt(nonce, input_data_as_bytes.as_ref())
        .map_err(|e| AppError::Internal(format!("Error in encrypting file: {}", e)))?;

    // Combine salt + nonce + ciphertext
    let mut output = Vec::new();
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypts data that was encrypted with `encrypt_file_with_password`.
///
/// # Arguments
/// * `encrypted_data` - The encrypted byte array containing salt + nonce + ciphertext.
/// * `password` - The password used to derive the decryption key.
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the decrypted plaintext.
/// * `Err(AppError)` if decryption or key derivation fails.
pub fn decrypt_file_with_password(
    encrypted_data: &[u8],
    password: &str,
) -> Result<Vec<u8>, AppError> {
    // Extract salt (first 16 bytes)
    let salt = encrypted_data
        .get(0..16)
        .ok_or_else(|| AppError::Internal("Missing salt".to_string()))?;

    // Extract nonce (next 12 bytes)
    let nonce_bytes = encrypted_data
        .get(16..28)
        .ok_or_else(|| AppError::Internal("Missing nonce".to_string()))?;

    // Extract ciphertext (remaining bytes)
    let ciphertext = encrypted_data
        .get(28..)
        .ok_or_else(|| AppError::Internal("Missing ciphertext".to_string()))?;

    // Derive the key using the same method as encryption
    let key_bytes = derive_key_from_password(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt and return plaintext
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| AppError::Internal(format!("Can not decrypt file: {}", e)))?;

    Ok(plaintext)
}

/// Saves an encrypted file to the local `./media` directory with a timestamped name.
///
/// # Arguments
/// * `file` - A reference to the encrypted file bytes.
/// * `file_name` - A base name to include in the output file name.
///
/// # Returns
/// * `Ok(String)` containing the file path of the saved file.
/// * `Err(AppError)` if the directory or file operation fails.
pub fn upload_file_to_server(file: &Vec<u8>, file_name: &str) -> Result<String, AppError> {
    let upload_dir = "./media";

    // Create the /uploads directory if it doesn't exist
    if !Path::new(upload_dir).exists() {
        fs::create_dir(upload_dir)
            .map_err(|e| AppError::Internal(format!("Error creating directory: {}", e)))?;
    }

    // Define the file path where the uploaded file will be saved
    let file_path = format!("{}/{}_{}.enc", upload_dir, file_name, Utc::now());

    // Create a file and write the content from the `file` bytes
    let mut file_out = File::create(&file_path)
        .map_err(|e| AppError::Internal(format!("Error creating file: {}", e)))?;

    file_out
        .write_all(&file)
        .map_err(|e| AppError::Internal(format!("Error writing to file: {}", e)))?;

    Ok(file_path)
}
