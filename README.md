# 📁 fileshare-rs

**fileshare-rs** is a secure, end-to-end encrypted file sharing web service built with [Rust](https://www.rust-lang.org/) and [Axum](https://github.com/tokio-rs/axum). Inspired by services like Firefox Send (RIP) and WeTransfer, it allows users to upload files and share secure download links with optional password protection, expiry, and download limits.

> 🔐 Designed with security and simplicity in mind — easily self-hostable and privacy-focused.

---

## 🧠 System Architecture

<img src="assets/auth_system.png" />
<img src="assets/file_system.png" />

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

# Create .env file
SERVER_URL=127.0.0.1:8000
MONGODB_URL=

SENDGRID_API_KEY=
SENDGRID_SENDER_NAME=fileshare-rs
SENDGRID_SENDER_EMAIL=

JWT_SECRET_KEY=my-jwt-secret-key

# Run the server
cargo run
```

## 📝 API Documentation

This outlines the available routes and their respective functionalities for the fileshare-rs.

---

## 📦 User Routes

Base Path: `/user`

### `POST /register`

Registers a new user.

- **Description**: Accepts user credentials to create a new account.
- **Request Body**: JSON object containing user data (e.g., email, password).
- **Response**: Success message or validation errors.

---

### `POST /send-verification-email`

Sends an email with a verification link to the user.

- **Description**: Initiates the email verification process.
- **Request Body**: JSON with user email.
- **Response**: Email sent confirmation.

---

### `POST /login`

Authenticates a user.

- **Description**: Logs in the user and returns a session or token.
- **Request Body**: JSON with email and password.
- **Response**: JSON with authentication token or error message.

---

### `GET /verify`

Verifies the user's email.

- **Description**: Confirms the user account via verification token.
- **Query Parameters**: Typically includes the verification token.
- **Response**: Success or failure message.

---

### `POST /forgot-password`

Sends a password reset link.

- **Description**: Triggers an email with instructions to reset the password.
- **Request Body**: JSON with user email.
- **Response**: Confirmation message.

---

### `PUT /reset-password`

Resets the user's password.

- **Description**: Uses a token to validate and update the password.
- **Request Body**: JSON with reset token and new password.
- **Response**: Password reset confirmation.

---

## 📁 File Routes

Base Path: `/file`

### 🔐 Protected Routes

> These routes require authentication via the `ExtractAuthAgent` middleware.

#### `POST /upload`

Uploads a file for the authenticated user.

- **Description**: Handles file uploads.
- **Request**: Multipart form data.
- **Headers**: `Authorization` token required.
- **Response**: Upload confirmation and file metadata.

---

#### `GET /user-files`

Lists all files uploaded by the authenticated user.

- **Description**: Fetches the authenticated user's files.
- **Headers**: `Authorization` token required.
- **Response**: JSON array of file metadata.

---

### 🌐 Public Routes

#### `POST /download`

Downloads a file.

- **Description**: Initiates file download. May require token or file identifier.
- **Request Body**: JSON with file ID or access token.
- **Response**: File stream or error message.

---

## ❤️ Health Check Route

Base Path: `/`

### `GET /`

Checks the server's health status.

- **Description**: Returns a simple status message to indicate the server is running.
- **Response**:
  ```json
  {
    "message": "Server is healthy!"
  }
  ```
