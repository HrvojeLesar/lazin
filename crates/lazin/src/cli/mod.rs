
use clap::{ArgAction, Parser, Subcommand};

use crate::{common, error::LazinResult};

pub mod check;
pub mod init;
pub mod list_workspaces;
pub mod version;

#[derive(Subcommand)]
enum Commands {
    Init(init::Init),
    Check(check::Check),
}

#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[arg(short = 'v', long = "version", action = ArgAction::SetTrue)]
    version: bool,
    #[command(subcommand)]
    commands: Option<Commands>,
    #[arg(short = 'q', long = "quiet", help = "Suppress all non error output")]
    quiet: bool,
    #[arg(
        short = 'l',
        long = "list-workspaces",
        help = "List configured workspace keys"
    )]
    list_workspaces: bool,
}

impl Cli {
    pub fn main() {
        let cli = Self::parse();

        if cli.version {
            version::version();
            common::exit_success();
        }
        if cli.list_workspaces {
            handle_error(list_workspaces::list_workspaces(None))
        }

        if cli.quiet {
            lazin_logger::quiet(true);
        }

        let result: LazinResult<()> = match cli.commands {
            Some(Commands::Init(cmd)) => cmd.init(),
            Some(Commands::Check(cmd)) => cmd.check(),
            None => Ok(()),
        };

        handle_error(result);
    }
}

fn handle_error<T>(result: LazinResult<T>) {
    // TODO: Better errors
    if let Err(e) = result {
        lazin_logger::error!(e);
        common::exit_error()
    }
}
