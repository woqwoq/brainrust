mod cli;
mod error;
mod interpreter;
mod program;
mod token;
mod tokenizer;

use crate::cli::CliRunner;

// TODO:
// 2. Add CLI
// 3. Add interactive Debug Mode
// 4. Comments?

fn main() {
    let mut cli_runner = CliRunner::new();

    cli_runner.run();
}
