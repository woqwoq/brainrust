mod cli;
mod error;
mod interpreter;
mod program;
mod token;
mod tokenizer;

use crate::cli::CliRunner;

// TODO:
// 4. Comments?

fn main() {
    let mut cli_runner = CliRunner::new();

    cli_runner.execute();
}
