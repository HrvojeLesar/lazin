use clap::{ArgAction, Parser, Subcommand};
use error::Error;

pub mod error;
pub mod init;
pub mod version;

#[derive(Subcommand)]
enum Commands {
    Init(init::Init),
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
}

impl Cli {
    pub fn main() {
        let cli = Self::parse();

        if cli.version {
            version::version()
        }

        if cli.quiet {
            lazin_logger::quiet(true);
        }

        let result: Result<(), Error> = match cli.commands {
            Some(Commands::Init(cmd)) => cmd.init(),
            None => Ok(()),
        };

        if let Err(e) = result {
            lazin_logger::error!("{}", e);
        }
    }
}
