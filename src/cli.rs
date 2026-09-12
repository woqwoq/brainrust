use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    process,
};

use clap::Parser;

use crate::{interpreter::Interpreter, program::Program};

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
        InterpreterConfig {
            memsize: DEFAULT_TAPE_SIZE,
        }
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
                    let _ = file.read_to_string(&mut code).unwrap();

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

        if self.cli.interactive_mode {
            self.run_interpreter_interactive(&code);
        } else {
            self.run_interpreter(&code);
        }
    }

    fn run_interpreter_interactive(&mut self, code: &str) {
        let interpreter_config = InterpreterConfig::from(&self.cli);
        let mut interpreter = Interpreter::new(code, interpreter_config, io::stdin(), io::stdout());
        let program = interpreter.get_program().clone();

        let mut high_water = 0;
        while interpreter.has_instruction() {
            let pc = interpreter.get_current_pc();
            high_water = high_water.max(interpreter.get_current_memory_pointer());

            println!("----------------------------------------------");
            println!("Program: \n{}", render_program(&program, pc));
            println!(
                "Memory: {}",
                render_memory(&interpreter.mem_dump(), high_water)
            );
            println!(
                "s: step / r: run the program until halt / p: print current memory value / q: quit"
            );

            let mut user_input = String::new();
            if let Err(e) = io::stdin().read_line(&mut user_input) {
                println!("Failed to read user input: {e}");
            }
            if let Some(c) = user_input.chars().next() {
                match c {
                    's' => match interpreter.step() {
                        Ok(_) => {}
                        Err(e) => {
                            println!("\nFatal error encountered: {}", e);
                            process::exit(1)
                        }
                    },
                    'r' => match interpreter.run(false) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("\nFatal error encountered: {}", e);
                            process::exit(1)
                        }
                    },
                    'p' => {
                        print!("Value at current cell: ");
                        let _ = interpreter.handle_output();
                        println!();
                    }
                    'q' => process::exit(0),
                    _ => {
                        println!("Unknown command: {c}.")
                    }
                }
            };
        }
        println!("\nFinished execution of the program.")
    }

    fn run_interpreter(&mut self, code: &str) {
        let interpreter_config = InterpreterConfig::from(&self.cli);

        let mut interpreter = Interpreter::new(code, interpreter_config, io::stdin(), io::stdout());

        match interpreter.run(false) {
            Ok(_) => println!("\nFinished execution of the program."),
            Err(e) => {
                println!("\nFatal error encountered: {}", e);
                process::exit(0)
            }
        };

        if self.cli.memdump {
            println!("Memory Dump: {:?}", interpreter.mem_dump());
        }
    }
}

fn render_program(program: &Program, pc: usize) -> String {
    format!("{program}\n{:>width$}^", "", width = pc)
}

fn render_memory(memory: &[u8], high_water: usize) -> String {
    let values = memory
        .iter()
        .take(high_water + 1)
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{values}]")
}
