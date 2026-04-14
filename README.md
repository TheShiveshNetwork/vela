## Vela

> Convert any storage device into your own local cloud

---

### Overview

Vela is a Rust CLI tool that lets you scan, mount, and serve any external storage device over your local network. Turn your USBs, SSDs, or other storage devices into a personal local cloud quickly.

---

### Setup

Clone the repository:

```bash
git clone https://github.com/<your-username>/vela.git
cd vela
```

Build and install locally (as `tvela`):

```bash
make install
```

This installs the binary as `tvela` in `~/.local/bin/`. Ensure this directory is in your `PATH`.

---

### CLI Commands

All commands can be run using the `tvela` binary after installation.

#### 1. Scan

List all storage devices connected to your system in a tree-like structure:

```bash
tvela scan
```

Output example:

```
NAME               LABEL      SIZE       TYPE  MNT NAME     MOUNT           REM   PERM
------------------ ---------- ---------- ----- ------------ --------------- ----- --------
/dev/sda           -          7.46GiB    disk  -            -               Yes   RW
└─sda1             FIREFLY    7.46GiB    part  temp         /mnt/temp       -     RW
```

---

#### 2. Mount

Mount a device at `/mnt/<mount_name>` (creates the folder if missing):

```bash
tvela mount <device> <mount_name>
```

* `device`: Path to the storage device (e.g., `/dev/sda1`)
* `mount_name`: Folder name under `/mnt/`.

Example:

```bash
tvela mount /dev/sda1 ext
```

Mounted at `/mnt/ext`.

---

#### 3. Unmount

Unmount a previously mounted device:

```bash
tvela unmount <mountpoint>
```

Example:

```bash
tvela unmount /mnt/ext
```

---

#### 4. Serve

Start an HTTP server serving a specific mount from `/mnt/` on the local network:

```bash
tvela serve <mount_name>
```

Example:

```bash
tvela serve ext
```

This will serve files from `/mnt/ext`.

Default server URL:

```
http://localhost:9000
```
