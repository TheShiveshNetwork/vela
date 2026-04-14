# Vela - Gemini CLI Context

This project, **Vela**, is a Rust-based CLI tool designed to convert any storage device into a personal local cloud. It handles device scanning, mounting, and serving files over a local network via an HTTP interface.

## 🎯 Project Core Objectives
- **Device Management**: Scan for block devices, mount them to `/mnt/`, and unmount them.
- **Web Service**: Serve a specific mount point from `/mnt/` over HTTP (port 9000) with a modern web interface for browsing and streaming media.

## 🛠 Tech Stack
- **Language**: Rust (Edition 2024)
- **Web Framework**: Axum
- **CLI Framework**: Clap (Derive API)
- **Async Runtime**: Tokio
- **Serialization**: Serde
- **Device Interaction**: `lsblk` crate and system `mount`/`umount` commands.

## 📁 Project Structure
- `src/main.rs`: Entry point, delegates to `cli.rs`.
- `src/cli.rs`: Command-line interface definitions and logic.
- `src/device.rs`: Logic for scanning, mounting, and unmounting devices.
- `src/server.rs`: Axum-based HTTP server for file browsing and streaming.
- `src/static/`: Frontend assets (HTML).

## 📜 Development Guidelines

### 1. Error Handling
- Use `anyhow::Result<()>` for top-level functions and library-style functions.
- Provide clear error messages for system-level operations (e.g., failed mounts).

### 2. File Serving & Streaming
- The server in `src/server.rs` supports HTTP Range requests for video streaming.
- Image viewing and PDF viewing are supported directly in the browser via the frontend.
- Large files are streamed using `tokio_util::io::ReaderStream`.

### 3. Frontend Architecture
- Single Page Application (SPA) in `src/static/index.html`.
- Communicates with `/api/list` and `/files/`.
- PDFs open in a new tab; images and videos open in a modal.

### 4. System Commands
- Mounting/unmounting require `sudo` permissions via `std::process::Command`.
- Mount points are managed under `/mnt/`.

### 5. Conventions
- Follow standard Rust naming conventions.
- Maintain separation between CLI, device logic, and server logic.

## 🚀 Common Commands
- **Build**: `cargo build`
- **Install**: `make install` (Installs as `tvela`)
- **Run**: `tvela <command>` (Commands: `scan`, `mount`, `unmount`, `serve`)
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## ⚠️ Known Constraints
- **Platform**: Designed for Linux.
- **Permissions**: Requires `sudo` for mounting operations.
- **Stateless**: The server does not persist selections; the mount name must be provided to the `serve` command.
