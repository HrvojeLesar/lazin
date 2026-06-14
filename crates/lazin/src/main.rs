use crate::cli::Cli;

mod cli;
mod common;
mod diagnostics;
mod dotfiles;
mod error;

#[cfg(test)]
mod test;

fn main() {
    Cli::main();
}
