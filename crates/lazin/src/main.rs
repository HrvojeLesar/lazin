use crate::cli::Cli;

mod cache;
mod cli;
mod common;
mod config;
mod encryption_management;
mod error;
mod filesystem;
#[cfg(unix)]
mod fingerprint;
mod resolve;

fn main() {
    Cli::main();
}
