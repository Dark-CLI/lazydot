use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(
    author,
    version = "v0.4.0",
    about = "Lazydot: CLI tool to manage and deploy your dotfiles efficiently",
    long_about = "Lazydot automates symlink creation for your configuration files, enabling consistent environments across multiple systems.",
    disable_help_subcommand = true
)]
pub struct LazyDotsArgs {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(long, hide = true)]
    pub completion_shell: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Register one or more dotfile paths in your config.
    #[clap(short_flag = 'a')]
    Add(AddArgs),

    /// Remove one or more paths from your config.
    #[clap(short_flag = 'r')]
    Remove(RemoveArgs),

    /// Create or update all symlinks according to the current config.
    #[clap(short_flag = 's')]
    Sync(SyncArgs),

    /// Unlink one or all paths without changing config.
    #[clap(short_flag = 'd')]
    DisableLink(DisableLinkArgs),

    /// Show what would be added or removed on next sync.
    #[clap(short_flag = 't')]
    Status(StatusArgs),

    /// Check the current state of each managed path.
    #[clap(short_flag = 'c')]
    Check(CheckArgs),
    /// Output shell completion script for a given shell.
    #[clap(short_flag = 'g', hide = true)]
    GenerateCompletion {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    
    /// Output man page to stdout.
    #[clap(hide = true)]
    GenerateMan,
}
#[derive(Debug, Args)]
pub struct StatusArgs {}

#[derive(Debug, Args)]
pub struct CheckArgs {}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Path to add (at least one required)
    #[arg(value_parser, required = true, num_args = 1..)]
    pub paths: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Path to remove (at least one required)
    #[arg(value_parser, required = true, num_args = 1..)]
    pub paths: Vec<String>,
}

#[derive(Debug, Args)]
pub struct SyncArgs {}

#[derive(Debug, Args)]
pub struct DisableLinkArgs {
    /// Unlink all managed symlinks
    #[clap(long = "all", short = 'a', action)]
    pub all: bool,

    /// Specific paths to unlink
    #[arg(value_parser, num_args = 1.., required_unless_present = "all")]
    pub paths: Vec<String>,
}
