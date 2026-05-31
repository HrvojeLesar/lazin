use crate::cli::Cli;

mod cli;
mod dotfiles;

#[cfg(test)]
mod test;

fn main() {
    Cli::main();
}
