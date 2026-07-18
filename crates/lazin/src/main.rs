use crate::cli::Cli;

mod cache;
mod cli;
mod common;
mod config;
mod dotfiles;
mod encryption_management;
mod error;
mod resolve;

#[cfg(test)]
mod test;

fn main() {
    Cli::main();
}
