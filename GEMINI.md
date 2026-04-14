# Vela - Gemini CLI Context

This project, **Vela**, is a Rust-based CLI tool designed to convert any storage device into a personal local cloud. It handles device scanning, mounting, selection, and serving files over a local network via an HTTP interface.

## 🎯 Project Core Objectives
- **Device Management**: Scan for block devices, mount them to `/mnt/`, and unmount them.
- **Selection Persistence**: Save a "selected" mount point to `storage-config.toml`.
- **Web Service**: Serve the selected mount point over HTTP (port 9000) with a modern web interface for browsing and streaming media.

## 🛠 Tech Stack
- **Language**: Rust (Edition 2024)
- **Web Framework**: Axum
- **CLI Framework**: Clap (Derive API)
- **Async Runtime**: Tokio
- **Serialization**: Serde, TOML
- **Device Interaction**: `lsblk` crate and system `mount`/`umount` commands.

## 📁 Project Structure
- `src/main.rs`: Entry point, delegates to `cli.rs`.
- `src/cli.rs`: Command-line interface definitions and logic.
- `src/device.rs`: Logic for scanning, mounting, and unmounting devices.
- `src/config.rs`: Configuration management for selected mount points.
- `src/server.rs`: Axum-based HTTP server for file browsing and streaming.
- `src/static/`: Frontend assets (HTML, JS, CSS).

## 📜 Development Guidelines

### 1. Error Handling
- Use `anyhow::Result<()>` for top-level functions and library-style functions where context is useful.
- Provide clear error messages for system-level operations (e.g., failed mounts).

### 2. File Serving & Streaming
- The server in `src/server.rs` supports HTTP Range requests for video streaming.
- Ensure `get_content_type` (in `server.rs`) is updated if new file types need specific MIME handling.
- Large files are streamed using `tokio_util::io::ReaderStream` to keep memory usage low.

### 3. Frontend Architecture
- The frontend is a Single Page Application (SPA) located in `src/static/index.html`.
- It communicates with `/api/list` for directory listings and `/files/` for downloading/streaming.
- Prefer keeping the frontend simple and self-contained within `src/static/`.

### 4. System Commands
- Mounting and unmounting require `sudo` permissions. The tool currently calls `sudo mount` and `sudo umount` via `std::process::Command`.
- Mount points are created under `/mnt/` by default.

### 5. Conventions
- Follow standard Rust naming conventions (`snake_case` for variables/functions, `PascalCase` for types).
- Maintain clear separation of concerns between CLI logic (`cli.rs`), system logic (`device.rs`), and web logic (`server.rs`).

## 🚀 Common Commands
- **Build**: `cargo build`
- **Run**: `cargo run -- <command>` (Commands: `scan`, `mount`, `unmount`, `select`, `serve`)
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## ⚠️ Known Constraints
- **Platform**: Currently designed for Linux systems (due to `/dev/`, `/mnt/`, and `lsblk` usage).
- **Permissions**: Requires the user to have `sudo` privileges for mounting operations.
