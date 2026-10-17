use crate::commands::virsh::virsh;
use anyhow::{Result, bail};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

fn find_ip(name: &str) -> Result<Option<String>> {
    let out = virsh(&["domifaddr", name])?;
    Ok(out
        .lines()
        .find(|l| l.contains("ipv4"))
        .and_then(|l| l.split_whitespace().nth(3))
        .and_then(|a| a.split('/').next())
        .map(str::to_owned))
}

pub(crate) fn start(
    name: String,
    user: String,
    private_key_file: &PathBuf,
    timeout: u64,
) -> Result<()> {
    if !private_key_file.exists() {
        bail!("key {:?} does not exist", private_key_file.display());
    }

    if virsh(&["domstate", &name])?.trim() != "running" {
        virsh(&["start", &name])?;
        println!("VM {name} has been started");
    }

    let deadline = Instant::now() + Duration::from_secs(timeout);
    let ip = loop {
        if let Some(ip) = find_ip(&name)? {
            break ip;
        }
        if Instant::now() > deadline {
            bail!("no IP found after {timeout}s");
        }
        std::thread::sleep(Duration::from_secs(2));
    };

    let addr: SocketAddr = format!("{ip}:22").parse()?;
    while TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_err() {
        if Instant::now() > deadline {
            bail!("ssh not reachable over {ip}");
        }
        std::thread::sleep(Duration::from_secs(2));
    }

    let status = Command::new("ssh")
        .arg("-i")
        .arg(private_key_file)
        .args(["-o", "IdentitiesOnly=yes"])
        .args(["-o", "StrictHostKeyChecking=no"])
        .args(["-o", "UserKnownHostsFile=/dev/null"])
        .args(["-o", "LogLevel=Error"])
        .arg(format!("{user}@{ip}"))
        .status()?;
    if !status.success() {
        bail!("ssh exited with {status}");
    }
    Ok(())
}
