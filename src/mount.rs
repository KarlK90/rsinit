// SPDX-License-Identifier: GPL-2.0-only
use crate::cmdline::CmdlineOptions;
use crate::Result;
use nix::mount::{mount, MsFlags};
use std::fs::{create_dir, remove_dir};
use std::io;
use std::path::Path;

pub fn setup_mountpoint(dir: &str) -> Result<()> {
    if let Err(e) = create_dir(dir) {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Err(format!("Failed to create {}: {e}", dir).into());
        }
    }
    Ok(())
}

pub fn do_mount(
    src: Option<&str>,
    dst: &str,
    fstype: Option<&str>,
    flags: MsFlags,
    data: Option<&str>,
) -> Result<()> {
    setup_mountpoint(dst)?;

    mount(src, dst, fstype, flags, data).map_err(|e| {
        format!(
            "Failed to mount {} -> {} ({:#x}, {}): {e}",
            src.unwrap_or(""),
            dst,
            flags.bits(),
            data.unwrap_or(""),
        )
    })?;

    Ok(())
}

fn mount_apivfs(dst: &str, fstype: &str) -> Result<()> {
    do_mount(
        Some(fstype),
        dst,
        Some(fstype),
        MsFlags::empty(),
        Option::<&str>::None,
    )?;

    Ok(())
}

pub fn mount_root(options: &CmdlineOptions) -> Result<()> {
    if options.root.is_none() {
        return Err("root= not found in /proc/cmdline".into());
    }

    if let Err(e) = create_dir("/root") {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Err(format!("Failed to create /root: {e}").into());
        }
    }

    println!(
        "Mounting rootfs {} -> /root ({}, '{}')",
        options.root.as_deref().unwrap(),
        options.rootfstype.as_deref().unwrap_or_default(),
        options.rootflags.as_deref().unwrap_or_default()
    );
    do_mount(
        options.root.as_deref(),
        "/root",
        options.rootfstype.as_deref(),
        options.rootfsflags,
        options.rootflags.as_deref(),
    )?;

    Ok(())
}

fn mount_move(src: &str, dst: &str, cleanup: bool) -> Result<()> {
    mount(
        Some(Path::new(src)),
        dst,
        Option::<&Path>::None,
        MsFlags::MS_MOVE,
        Option::<&Path>::None,
    )
    .map_err(|e| format!("Failed to move mount {src} -> {dst}: {e}"))?;

    if cleanup {
        remove_dir(src)?;
    }

    Ok(())
}

pub fn mount_special(mount_config: bool) -> Result<()> {
    mount_apivfs("/dev", "devtmpfs")?;
    mount_apivfs("/sys", "sysfs")?;
    if mount_config && Path::new("/sys/kernel/config").is_dir() {
        mount_apivfs("/sys/kernel/config", "configfs")?;
    }
    mount_apivfs("/proc", "proc")?;
    Ok(())
}

pub fn mount_move_special(options: &CmdlineOptions) -> Result<()> {
    mount_move("/dev", "/root/dev", options.cleanup)?;
    mount_move("/sys", "/root/sys", options.cleanup)?;
    mount_move("/proc", "/root/proc", options.cleanup)?;
    Ok(())
}
