use crate::cli::Cli;

mod cli;
mod diagnostics;
mod dotfiles;

#[cfg(test)]
mod test;

fn main() {
    Cli::main();
}
