use anyhow::{Result, anyhow};
use lsblk::BlockDevice;
use std::fs;
use std::process::Command;
use std::path::Path;

pub fn mount_device(device: &str, mount_name: &str) -> Result<()> {
    let mountpoint = format!("/mnt/{}", mount_name);

    println!("Mounting: {} -> {}", device, mountpoint);

    let path = Path::new(&mountpoint);
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("Created mountpoint: {}", mountpoint);
    }

    let status = Command::new("sudo")
        .arg("mount")
        .arg(device)
        .arg(&mountpoint)
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to mount device"));
    }

    println!("Mounted successfully at {}!", mountpoint);
    Ok(())
}

pub fn unmount_device(mountpoint: &str) -> Result<()> {
    println!("Unmounting: {}", mountpoint);

    let status = Command::new("sudo")
        .arg("umount")
        .arg(mountpoint)
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to unmount device"));
    }

    println!("Unmounted successfully!");
    Ok(())
}

pub fn scan() -> Result<()> {
    let devs = BlockDevice::list()?;
    println!("Storage devices found: ");

    for dev in devs {
        let path = dev.fullname.display().to_string();
        let uuid = dev.uuid.unwrap_or_else(|| "-".into());
        let label = dev.label.unwrap_or_else(|| "-".into());
        // let size = dev.size.unwrap_or_default();
        println!(
            "{:<25} {:<20} {}",
            path,
            label,
            uuid
        );
    }

    Ok(())
}

