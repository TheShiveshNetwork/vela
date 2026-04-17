use anyhow::{Result, anyhow};
use lsblk::{BlockDevice, mountpoints::Mount};
use std::fs;
use std::process::Command;
use std::path::Path;
use std::collections::HashMap;

use nix::unistd::{getuid, getgid};

pub fn mount_device(device: &str, mount_name: &str, read_only: bool) -> Result<()> {
    let mountpoint = format!("/mnt/{}", mount_name);
    let uid = getuid().as_raw();
    let gid = getgid().as_raw();

    println!("Mounting ({}): {} -> {}", if read_only { "RO" } else { "RW" }, device, mountpoint);

    let path = Path::new(&mountpoint);
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("Created mountpoint: {}", mountpoint);
    }

    let mut cmd = Command::new("sudo");
    cmd.arg("mount");
    
    let mut options = if read_only { "ro".to_string() } else { "rw".to_string() };
    
    // Add uid and gid to options to ensure the current user has permissions
    // Note: Some filesystems (like ext4) ignore these options, but they are 
    // essential for FAT, NTFS, exFAT, etc.
    options.push_str(&format!(",uid={},gid={}", uid, gid));
    
    cmd.arg("-o").arg(options);
    
    let status = cmd.arg(device)
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
    let mounts: HashMap<String, String> = Mount::list()?
        .map(|m| (m.device, m.mountpoint.display().to_string()))
        .collect();
    
    // Separate disks and partitions, filtering out loop devices
    let mut disks: Vec<BlockDevice> = devs.iter()
        .filter(|d| d.is_disk() && !d.name.starts_with("loop"))
        .cloned()
        .collect();
    let parts: Vec<BlockDevice> = devs.iter()
        .filter(|d| d.is_part() && !d.name.starts_with("loop"))
        .cloned()
        .collect();

    // Sort disks for consistent output
    disks.sort_by(|a, b| a.name.cmp(&b.name));

    // Updated columns for better fit with MOUNT NAME
    println!("{:<18} {:<10} {:<10} {:<5} {:<12} {:<15} {:<5} {}", 
             "NAME", "LABEL", "SIZE", "TYPE", "MNT NAME", "MOUNT", "REM", "PERM");
    println!("{:-<18} {:-<10} {:-<10} {:-<5} {:-<12} {:-<15} {:-<5} {:-<8}", 
             "", "", "", "", "", "", "", "");

    for disk in disks {
        print_device(&disk, "", false, &mounts);
        
        // Find partitions for this disk
        let mut disk_parts: Vec<BlockDevice> = parts.iter()
            .filter(|p| {
                if let Ok(parent_name) = p.disk_name() {
                    parent_name == disk.name
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        
        // Sort partitions
        disk_parts.sort_by(|a, b| a.name.cmp(&b.name));
        
        let count = disk_parts.len();
        for (i, part) in disk_parts.into_iter().enumerate() {
            let is_last = i == count - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            print_device(&part, prefix, true, &mounts);
        }
    }

    Ok(())
}

fn print_device(dev: &BlockDevice, prefix: &str, is_part: bool, mounts: &HashMap<String, String>) {
    let display_name = if is_part {
        format!("{}{}", prefix, dev.name)
    } else {
        dev.fullname.display().to_string()
    };

    let label = dev.label.as_deref().unwrap_or("-");
    let size = dev.capacity().ok().flatten().map(|c| format_size(c)).unwrap_or_else(|| "-".into());
    let dev_type = if is_part { "part" } else { "disk" };
    
    let mountpoint = mounts.get(&dev.fullname.display().to_string())
        .or_else(|| mounts.get(&format!("/dev/{}", dev.name)))
        .map(|s| s.as_str())
        .unwrap_or("-");

    // Extract mount name from /mnt/ paths
    let mount_name = if mountpoint.starts_with("/mnt/") {
        mountpoint.trim_start_matches("/mnt/").to_string()
    } else {
        "-".to_string()
    };

    // Try to get removable and read-only status from sysfs
    let mut removable = "-";
    let mut permissions = "RW";

    if let Ok(sysfs) = dev.sysfs() {
        if let Ok(remov_str) = fs::read_to_string(sysfs.join("removable")) {
            removable = if remov_str.trim() == "1" { "Yes" } else { "No" };
        }
        if let Ok(ro_str) = fs::read_to_string(sysfs.join("ro")) {
            if ro_str.trim() == "1" {
                permissions = "RO";
            }
        }
    }

    println!("{:<18} {:<10} {:<10} {:<5} {:<12} {:<15} {:<5} {}", 
             display_name, 
             truncate(label, 10), 
             size, 
             dev_type, 
             mount_name,
             truncate(mountpoint, 15), 
             removable, 
             permissions);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}..", &s[..max-2])
    } else {
        s.to_string()
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2}TiB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2}GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2}MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2}KiB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
