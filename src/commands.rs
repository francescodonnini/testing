use clap::Subcommand;
use std::path::PathBuf;

pub(crate) mod create;
pub(crate) mod start;
pub(crate) mod virsh;

const URI: &str = "qemu:///system";

fn default_vcpus() -> u32 {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    (threads / 2).max(1) as u32
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Create {
        name: String,
        #[arg(long)]
        disk: PathBuf,
        #[arg(long)]
        public_key: PathBuf,
        #[arg(long)]
        os_variant: String,
        #[arg(long)]
        user_data: PathBuf,
        #[arg(long)]
        mem: u32,
        #[arg(long, short='n', default_value_t = default_vcpus())]
        vcpus: u32,
    },
    Start {
        name: String,
        #[arg(long, short = 'U')]
        user: String,
        #[arg(long)]
        private_key: PathBuf,
        #[arg(long, default_value_t = 120)]
        timeout: u64,
    },
    Stop {
        name: String,
        #[arg(long)]
        force: bool,
    },
}
