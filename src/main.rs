mod args;
mod config;
mod dot_manager;
mod tests {
    pub mod test_dot_manager;
    pub mod test_utils;
}
mod create_toml_temp;
mod current_state;
mod utils;

use crate::args::Command;
use crate::config::OnDelinkBehavior;
use crate::dot_manager::DotManager;
use args::LazyDotsArgs;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use config::Config;
use std::io::{self, Write};

fn main() {
    let args = LazyDotsArgs::parse();
    // Handle shell completion generation
    if let Some(shell) = args.completion_shell {
        let mut cmd = LazyDotsArgs::command();
        let shell: Shell = shell.parse().expect("Invalid shell type");
        generate(shell, &mut cmd, "lazydot", &mut io::stdout());
        return;
    }

    match args.command {
        Command::Add(add_args) => {
            let mut config = Config::new();
            for path in add_args.paths {
                config.add_path(path).expect("failed to add path");
            }
        }
        Command::Remove(remove_args) => {
            let mut config = Config::new();
            for path in remove_args.paths {
                config.remove_path(path);
            }
        }
        Command::Sync(_apply_args) => {
            let manager = DotManager::new();
            manager.sync();
        }
        Command::GenerateCompletion { shell } => {
            let mut cmd = LazyDotsArgs::command();
            generate(shell, &mut cmd, "lazydot", &mut io::stdout());
        }
        Command::DisableLink(delink_args) => {
            let mut manager = DotManager::new();
            match delink_args.all {
                true => {
                    manager.config.defaults.on_delink = OnDelinkBehavior::Keep;
                    manager.delink_all();
                }
                false => {
                    manager.delink(&delink_args.paths);
                }
            }
        }
        Command::Status(_) => {
            let manager = DotManager::new();
            manager.status();
        }
        Command::Check(_) => {
            let manager = DotManager::new();
            manager.check();
        }
        Command::GenerateMan => {
            const MAN_PAGE: &str = include_str!("../man/lazydot.1");
            io::stdout().write_all(MAN_PAGE.as_bytes()).expect("Failed to write man page");
        }
    }
}