use crate::commands::URI;
use anyhow::{Context, bail};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub(crate) fn create(
    name: String,
    disk: &PathBuf,
    os_variant: String,
    public_key_file: &PathBuf,
    user_data: &PathBuf,
    mem: u32,
    vcpus: u32,
) -> anyhow::Result<()> {
    if let Some(&f) = [disk, public_key_file, user_data]
        .iter()
        .find(|&p| !p.exists())
    {
        bail!("file {} not found", f.display());
    }

    let pass = match std::env::var("VM_PASS") {
        Ok(p) => p,
        Err(_) => rpassword::prompt_password("passwd: ")?,
    };
    let hash = pwhash::sha512_crypt::hash(&pass)?;
    let pubkey = std::fs::read_to_string(&public_key_file)
        .with_context(|| format!("reading public key {}", public_key_file.display()))?;
    let yaml = std::fs::read_to_string(user_data)
        .with_context(|| format!("reading user data {}", user_data.display()))?
        .replace("__SSH_PUBKEY__", pubkey.trim())
        .replace("__PASSWD_HASH__", &hash);
    let mut tmp = tempfile::Builder::new()
        .suffix(".yaml")
        .tempfile()
        .context("creating temporary user-data file")?;
    tmp.write_all(yaml.as_bytes())?;
    let status = Command::new("virt-install")
        .args(["--connect", URI])
        .args(["--name", &name])
        .args(["--memory", &mem.to_string()])
        .args(["--vcpus", &vcpus.to_string()])
        .args(["--cpu", "host-passthrough", "--import"])
        .arg("--disk")
        .arg(&disk)
        .args(["--os-variant", &os_variant])
        .arg("--cloud-init")
        .arg(format!("user-data={}", tmp.path().display()))
        .args(["--network", "network=default"])
        .args(["--graphics", "none", "--console", "pty,target_type=serial"])
        .arg("--noautoconsole")
        .status()
        .context("running virt-install")?;
    if !status.success() {
        bail!("virt-install failed");
    }
    Ok(())
}
