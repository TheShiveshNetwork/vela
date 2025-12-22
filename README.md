## Vela

> Convert any storage device into your own local cloud

---

### Overview

Vela is a Rust CLI tool that lets you scan, mount, select, and serve any external storage device over your local network. Turn your USBs, SSDs, or other storage devices into a personal local cloud quickly.

---

### Setup

Clone the repository:

```bash
git clone https://github.com/<your-username>/vela.git
cd vela
```

Build the project:

```bash
cargo build --release
```

Run the project:

```bash
cargo run -- <command>
```

---

### CLI Commands

#### 1. Scan

List all storage devices connected to your system:

```bash
cargo run -- scan
```

Output example:

```
/dev/sda1           EXTSSD      1234-ABCD
/dev/sdb1           USBDRIVE    5678-EFGH
```

---

#### 2. Mount

Mount a device at `/mnt/<mount_name>` (creates the folder if missing):

```bash
cargo run -- mount <device> [mount_name]
```

* `device`: Path to the storage device (e.g., `/dev/sda1`)
* `mount_name`: Optional; folder name under `/mnt`. If omitted, the device name is used.

Example:

```bash
cargo run -- mount /dev/sda1 ext
```

Mounted at `/mnt/ext`.

---

#### 3. Unmount

Unmount a previously mounted device:

```bash
cargo run -- unmount <mountpoint>
```

Example:

```bash
cargo run -- unmount /mnt/ext
```

---

#### 4. Select

Select a mounted path to serve over the network:

```bash
cargo run -- select <mount_path>
```

Example:

```bash
cargo run -- select /mnt/ext
```

This saves the selection for the `serve` command.

---

#### 5. Serve

Start an HTTP server serving the selected storage path on the local network:

```bash
cargo run -- serve
```

Default server URL:

```
http://localhost:9000
```

