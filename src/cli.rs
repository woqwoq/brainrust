use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::Parser;

use crate::interpreter::Interpreter;

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
        default_value_t = 30000
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
                panic!("File '{}' does not exist.", file_path.to_str().unwrap())
            }

            match File::open(file_path) {
                Ok(mut file) => {
                    let s = file.read_to_string(&mut code).unwrap();
                    println!("file opened {s}");

                    if code.is_empty() {
                        panic!(
                            "Error: The input file '{}' is empty.",
                            file_path.to_str().unwrap()
                        )
                    }
                }
                Err(e) => panic!(
                    "Failed to open file '{}' with error: {}",
                    file_path.to_str().unwrap(),
                    e
                ),
            }
        }

        if let Some(inline_code) = &self.cli.inline_program {
            code = inline_code.clone();
        }

        self.run_interpreter(&code);
    }

    fn run_interpreter(&mut self, code: &str) {
        let mut interpreter = Interpreter::new(code, io::stdin(), io::stdout());

        match interpreter.run(false) {
            Ok(_) => println!("Finished execution of the program."),
            Err(e) => println!("Fatal error encountered: {}", e),
        };
    }
}
