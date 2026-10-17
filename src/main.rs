pub mod commands;

use crate::commands::Command;
use crate::commands::create::create;
use crate::commands::start::start;
use crate::commands::virsh::virsh;
use clap::Parser;

#[derive(Parser)]
#[command(name = "vmctl")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Create {
            name,
            disk,
            public_key: pubkey,
            os_variant,
            user_data,
            mem,
            vcpus,
        } => create(name, &disk, os_variant, &pubkey, &user_data, mem, vcpus),
        Command::Start {
            name,
            user,
            private_key: key,
            timeout,
        } => start(name, user, &key, timeout),
        Command::Stop { name, force } => {
            virsh(&[if force { "destroy" } else { "shutdown" }, &name]).map(|_| {})
        }
    }
}
