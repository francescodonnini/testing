use crate::commands::URI;
use anyhow::{Context, bail};
use std::process::Command;

pub(crate) fn virsh(args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("virsh")
        .args(["--connect", URI])
        .args(args)
        .output()
        .context("cannot execute virsh")?;
    if !out.status.success() {
        bail!(
            "virsh {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
