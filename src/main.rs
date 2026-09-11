mod cli;
mod interpreter;
mod program;
mod token;
mod tokenizer;

use std::io;

use clap::Parser;
use interpreter::Interpreter;

use crate::cli::Cli;

// TODO:
// 2. Add CLI
// 3. Add interactive Debug Mode
// 4. Comments?

fn main() {
    let cli = Cli::parse();

    let code = "";
    let mut bf = Interpreter::new(code, io::stdin(), io::stdout());

    let _ = bf.run(false);
}
