use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::Parser;

use crate::interpreter::Interpreter;

const DEFAULT_TAPE_SIZE: usize = 500;

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
    #[arg(
        short = 'd',
        long,
        value_name = "IMODE",
        requires = "input",
        default_value_t = false
    )]
    pub interactive_mode: bool,

    /// Optional parameter to set the memory tape size
    #[arg(
        long,
        value_name = "MEMSIZE",
        requires = "input",
        default_value_t = DEFAULT_TAPE_SIZE
    )]
    pub memsize: usize,

    /// Dump memory at the end of the program execution
    #[arg(
        long,
        value_name = "MEMDUMP",
        requires = "input",
        default_value_t = false
    )]
    pub memdump: bool,
}

pub struct InterpreterConfig {
    pub memsize: usize,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        InterpreterConfig { memsize: 30000 }
    }
}

impl InterpreterConfig {
    pub fn from(cli: &Cli) -> Self {
        InterpreterConfig {
            memsize: cli.memsize,
        }
    }
}

pub struct CliRunner {
    cli: Cli,
}

impl CliRunner {
    pub fn new() -> Self {
        CliRunner { cli: Cli::parse() }
    }

    pub fn run(&mut self) {
        let mut code = String::new();
        if let Some(file_path) = &self.cli.file {
            if !file_path.exists() {
                panic!(
                    "Error: File '{}' does not exist.",
                    file_path.to_str().unwrap()
                )
            }

            match File::open(file_path) {
                Ok(mut file) => {
                    let s = file.read_to_string(&mut code).unwrap();

                    if code.is_empty() {
                        panic!("Error: File '{}' is empty.", file_path.to_str().unwrap())
                    }
                }
                Err(e) => panic!(
                    "Error: Failed to open File '{}' with error: {}",
                    file_path.to_str().unwrap(),
                    e
                ),
            }
        }

        if let Some(inline_code) = &self.cli.inline_program {
            code = inline_code.clone();
        }

        let interpreter_config = InterpreterConfig::from(&self.cli);
        self.run_interpreter(&code, interpreter_config);
    }

    fn run_interpreter(&mut self, code: &str, interpreter_config: InterpreterConfig) {
        let mut interpreter = Interpreter::new(code, interpreter_config, io::stdin(), io::stdout());

        match interpreter.run(false) {
            Ok(_) => println!("\nFinished execution of the program."),
            Err(e) => println!("\nFatal error encountered: {}", e),
        };
    }
}
