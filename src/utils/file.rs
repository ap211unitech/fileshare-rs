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

// Derives a 256-bit AES key from a user password and random salt using Argon2id
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

// Encrypts a file with a user-given password and writes salt + nonce + ciphertext to output
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

    // Write the salt, nonce, and encrypted data to output file
    // let mut file = File::create(output_path)?;
    // file.write_all(&salt)?; // First 16 bytes: salt
    // file.write_all(&nonce_bytes)?; // Next 12 bytes: nonce
    // file.write_all(&ciphertext)?; // Remaining: encrypted content

    // Combine salt + nonce + ciphertext
    let mut output = Vec::new();
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

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
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

// pub fn main() {
//     let input_file_content = fs::read_to_string("./hashing.rs").unwrap();

//     let user_password = "12345";

//     encrypt_file_with_password(input_file_content.as_bytes().to_vec(), user_password).unwrap();

//     println!("{}", input_file_content);
// }

pub fn upload_file_to_server(file: &Vec<u8>, file_name: &str) -> Result<String, AppError> {
    // let input_file_content = fs::read_to_string("/Users/arjunporwal/Documents/Rust/fileshare-rs/src/utils/hashing.rs").expect("here is the error");

    // let user_password = "12345";

    // encrypt_file_with_password(input_file_content.as_bytes().to_vec(), user_password).unwrap();

    // // println!("{}", input_file_content);

    // return Ok("()".to_string());

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
