# 🎵 Music Platform Backend

This is the backend for the Music Platform built using Axum, Rust, SQLx, WebSockets, HTTP/2, and validation libraries.

## 🚀 Features
- **Axum** – High-performance web framework for Rust
- **SQLx** – Async database interactions with PostgreSQL
- **WebSockets** – Real-time communication support
- **HTTP/2** – Improved performance with multiplexing
- **Validation** – Data validation to ensure request integrity

## 📦 Tech Stack
- **Framework:** Axum (Rust)
- **Database:** SQLx (PostgreSQL)
- **Real-time:** WebSockets
- **Protocol:** HTTP/2
- **Validation:** Custom validation library

## 🛠️ Setup & Installation
### Prerequisites
- Rust (latest stable)
- PostgreSQL database

### Installation
1. Clone the repository:
   ```sh
   git clone https://github.com/AarambhDevHub/music-pathform-backend.git
   cd music-platform-backend
   ```
2. Install dependencies:
   ```sh
   cargo build
   ```
3. Configure environment variables in `.env`:
   ```sh
   # -----------------------------------------------------------------------------
   # Database (PostgreSQL)
   # -----------------------------------------------------------------------------
   DATABASE_URL=postgresql://username:password@localhost:5432/music

   # -----------------------------------------------------------------------------
   # JSON Web Token Credentials
   # -----------------------------------------------------------------------------
   JWT_SECRET_KEY=my_ultra_secure_jwt_secret_key
   JWT_MAXAGE=60
   ```
4. Run database migrations:
   ```sh
   sqlx migrate run
   ```
5. Start the server:
   ```sh
   cargo run
   ```

## 🔌 WebSocket Support
This backend includes WebSocket support for real-time features like live music streaming and chat.

## 📡 Deployment
To build for production:
```sh
cargo build --release
```
Deploy on **Docker**, **AWS**, or your preferred cloud provider.

## 📜 License
This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.
---

## Donations

If you find this project useful and would like to support its continued development, you can make a donation via [Buy Me a Coffee](https://buymeacoffee.com/aarambhdevhub).

Thank you for your support!

Happy coding! 🎧🚀
