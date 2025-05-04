# 📁 fileshare-rs

**fileshare-rs** is a secure, end-to-end encrypted file sharing web service built with [Rust](https://www.rust-lang.org/) and [Axum](https://github.com/tokio-rs/axum). Inspired by services like Firefox Send (RIP) and WeTransfer, it allows users to upload files and share secure download links with optional password protection, expiry, and download limits.

> 🔐 Designed with security and simplicity in mind — easily self-hostable and privacy-focused.

---

## 🧠 System Architecture

<img src="assets/auth_system.png" />
<img src="assets/auth_system.png" />

## 🚀 Features

### 🔒 Authentication (JWT + Email Verification)

- User registration & login
- Secure password hashing (Argon2)
- Email verification workflow
- Password reset with email token

### 📤 File Upload

- Supports multipart/form-data for file uploads
- File size validation
- Metadata stored in MongoDB: filename, size, hash, etc.
- File content saved securely to local disk

### 🔗 Secure Download Links

- File download link generation
- Optional password protection (Argon2 hashed)
- Set max download count and expiry time
- Token-based access control

### 🧹 Expiry & Cleanup

- Time-based expiry checks
- Automatic deletion of expired files or after max downloads
- Background job or cron cleanup support

---

## ⚙️ Tech Stack

| Layer      | Tech                      |
| ---------- | ------------------------- |
| Language   | Rust                      |
| Framework  | Axum                      |
| Database   | MongoDB                   |
| Storage    | Local filesystem          |
| Auth       | JWT, Argon2, Email Tokens |
| Background | Tokio + Cron Tasks        |

---

## 🛠️ Setup

### Prerequisites

- [Rust](https://rustup.rs/)
- [MongoDB](https://www.mongodb.com/)
- `cargo`, `openssl`, and `npm` (optional, for UI/client)

### Running Locally

```bash
# Clone the repo
git clone https://github.com/yourusername/fileshare-rs.git
cd fileshare-rs

# Run the server
cargo run

```