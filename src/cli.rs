use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(about = "A CLI Brainfuck interpreter.")]
pub struct Cli {
    /// Path to program file
    #[arg(short, long, value_name = "FILE", group = "input")]
    pub file: Option<PathBuf>,

    /// Run BF code directly from user's input
    #[arg(short, long, value_name = "CODE", group = "input")]
    pub inline_program: Option<String>,

    /// Optional parameter to enable interactive step-by-step mode (DEBUG)
    #[arg(short = 'd', long, value_name = "IMODE", requires = "input")]
    pub interactive_mode: Option<bool>,

    /// Optional parameter to set the memory tape size
    #[arg(
        long,
        value_name = "MEMSIZE",
        requires = "input",
        default_value_t = 30000
    )]
    pub memsize: usize,

    /// Dump memory at the end of the program execution
    #[arg(long, value_name = "MEMDUMP", requires = "input")]
    pub memdump: Option<bool>,
}
