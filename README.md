# Vela

> Convert any storage device into your own personal local cloud with ease.

Vela is a Rust-based CLI tool designed to scan, mount, and serve external storage devices over your local network via a modern web interface.

---

### ⚠️ CRITICAL SECURITY CHECKLIST
> **Read before running any commands:**
- 🛡️ **PRIVILEGE SAFETY:** Always run `tvela mount` with `sudo`, but run `tvela serve` as a **regular user** to prevent system-wide compromise.
- 🔒 **HARDWARE PROTECTION:** Use Read-Only mode (`-m r`) by default; it locks the physical drive at the OS kernel level.
- 📡 **DATA ENCRYPTION:** Plain HTTP is vulnerable to sniffing; always use **ngrok** for HTTPS when accessing your cloud remotely.
- 🔑 **SESSION SECURITY:** Vela uses `HttpOnly` cookies; your access tokens are never exposed to malicious browser scripts (XSS).

---

### Features

- 📂 **Device Management**: Scan for block devices and partitions.
- 🚀 **One-Command Cloud**: Mount and serve your files in seconds.
- 🛡️ **Defense in Depth**: OS-level kernel locks and application-level path traversal protection.
- 🔐 **Secure Authentication**: Automatic UUID v4 token generation and `HttpOnly` cookie sessions.
- 📤 **File Uploads**: Modern UI for uploading files to your cloud (when in write mode).
- 🎥 **Media Streaming**: Support for video seeking (Range requests) and in-browser previews.

---

### Installation

```bash
git clone https://github.com/shivu/vela.git
cd vela
make install
```

---

### CLI Usage

#### 1. Scan Devices
```bash
tvela scan
```

#### 2. Mount a Device
```bash
# Mount as Read-Only (Secure Default)
sudo tvela mount /dev/sdb1 mydrive -m r

# Mount as Read-Write
sudo tvela mount /dev/sdb1 mydrive -m w
```

#### 3. Serve the Cloud
```bash
# Start the server (DO NOT run as sudo)
tvela serve mydrive -m r
```
**Options:**
- `-m, --mode <MODE>`: `r` (Read-Only) or `w` (Read-Write/Uploads).
- `--no-auth`: Disable authentication (only for trusted local tests).

---

### Accessing your Cloud

#### Local & LAN Access
- **Local**: `http://localhost:9000`
- **LAN**: `http://<your-ip>:9000`

#### Remote Access via ngrok (Recommended)
To expose your local cloud to the internet securely with HTTPS encryption:
```bash
# Replace <your-ip> with your local LAN IP (e.g., 192.168.1.5)
ngrok http <your-ip>:9000
```
*Vela's cookie-based authentication is fully compatible with ngrok's secure TLS-terminated tunnels.*

---

### Tech Stack
- **Backend**: Rust (Axum, Tokio, Tower-Cookies)
- **Frontend**: Vanilla JS / CSS (SPA with Blob-streaming support)
- **System**: Linux `mount` with UID/GID mapping and `lsblk`
