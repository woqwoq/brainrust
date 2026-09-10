use std::collections::HashMap;
use std::io::Write;
use std::{fmt, io};

use crate::token::Token;
use crate::tokenizer::{JumpTable, Tokenizer};

const DEFAULT_TAPE_SIZE: usize = 500;

pub struct Interpreter {
    program_counter: usize,
    instructions: Vec<Token>,
    memory_pointer: usize,
    memory: [u8; DEFAULT_TAPE_SIZE],
    jump_table: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn new(instructions: &str) -> Self {
        let tokens = Tokenizer::tokenize(instructions);
        Interpreter {
            program_counter: 0,
            instructions: tokens.clone(),
            memory_pointer: 0,
            memory: [0u8; DEFAULT_TAPE_SIZE],
            jump_table: JumpTable::from(&tokens),
        }
    }

    pub fn handle_move_left(&mut self) {
        if self.memory_pointer == 0 {
            panic!(
                "Memory tape pointer left allowed bounds on pc={}",
                self.program_counter
            )
        }

        self.memory_pointer -= 1;
    }

    pub fn handle_move_right(&mut self) {
        if self.memory_pointer + 1 >= DEFAULT_TAPE_SIZE {
            panic!(
                "Memory tape pointer left allowed bounds on pc={}",
                self.program_counter
            )
        }

        self.memory_pointer += 1;
    }

    pub fn handle_increment(&mut self) {
        let memory_value = self.fetch_memory_mut(self.memory_pointer);
        *memory_value = memory_value.wrapping_add(1);
    }

    pub fn handle_decrement(&mut self) {
        let memory_value = self.fetch_memory_mut(self.memory_pointer);
        *memory_value = memory_value.wrapping_sub(1);
    }

    pub fn handle_output(&mut self) {
        let memory_val_ascii =
            char::from_u32(*self.fetch_memory(self.memory_pointer) as u32).unwrap();
        print!("{}", memory_val_ascii);
        io::stdout().flush().unwrap();
    }

    pub fn handle_input(&mut self) {
        let mut string_buffer = String::new();

        io::stdin().read_line(&mut string_buffer).unwrap();

        *self.fetch_memory_mut(self.memory_pointer) = string_buffer.bytes().next().unwrap();
    }

    fn jump_to_closest_bracket(&mut self) {
        self.program_counter = *self.jump_table.get(&self.program_counter).unwrap();
    }

    pub fn handle_loop_start(&mut self) {
        let memory_value = self.fetch_memory(self.memory_pointer);

        if *memory_value == 0 {
            self.jump_to_closest_bracket();
        }
    }

    pub fn handle_loop_close(&mut self) {
        let memory_value = self.fetch_memory(self.memory_pointer);

        if *memory_value > 0 {
            self.jump_to_closest_bracket();
        }
    }

    pub fn fetch_instruction(&self) -> Option<Token> {
        self.instructions.get(self.program_counter).cloned()
    }

    pub fn fetch_memory(&self, index: usize) -> &u8 {
        if self.memory.len() <= index {
            panic!(
                "Trying to access memory at cell={} while DEFAULT_TAPE_SIZE={}",
                index, DEFAULT_TAPE_SIZE
            )
        }

        if let Some(memory_value) = self.memory.get(index) {
            memory_value
        } else {
            panic!(
                "Cannot acces value at tape position={} during executing instruction at pc={}",
                self.memory_pointer, self.program_counter
            )
        }
    }

    pub fn fetch_memory_mut(&mut self, index: usize) -> &mut u8 {
        if self.memory.len() <= index {
            panic!(
                "Trying to access memory at cell={} while DEFAULT_TAPE_SIZE={}",
                index, DEFAULT_TAPE_SIZE
            )
        }

        if let Some(memory_value) = self.memory.get_mut(index) {
            memory_value
        } else {
            panic!(
                "Cannot acces value at tape position={} during executing instruction at pc={}",
                self.memory_pointer, self.program_counter
            )
        }
    }

    pub fn execute_instruction(&mut self, instruction: Token) {
        match instruction {
            Token::MoveLeft => self.handle_move_left(),
            Token::MoveRight => self.handle_move_right(),
            Token::Increment => self.handle_increment(),
            Token::Decrement => self.handle_decrement(),
            Token::Output => self.handle_output(),
            Token::Input => self.handle_input(),
            Token::LoopStart => self.handle_loop_start(),
            Token::LoopClose => self.handle_loop_close(),
            Token::Comment(_) => {}
        }
    }

    pub fn step(&mut self) {
        if let Some(instruction) = self.fetch_instruction() {
            self.execute_instruction(instruction);
            self.program_counter += 1
        } else {
            panic!("Failed to fetch instruction at pc={}", self.program_counter) // not expected
        }
    }

    pub fn run(&mut self, mem_dump: bool) {
        while self.instructions.len() > self.program_counter {
            self.step();
        }
        if mem_dump {
            println!("{:?}", self.memory);
        }
    }
}

impl fmt::Display for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "---\nPC: {}\nInstruction: {:?}\nMemory Pointer: {}\nMemory Dump: {:?}",
            self.program_counter,
            self.instructions[self.program_counter],
            self.memory_pointer,
            self.memory
        )
    }
}

#[cfg(test)]
mod interpreter_tests {
    use std::io::{self, Read};

    use crate::interpreter::{DEFAULT_TAPE_SIZE, Interpreter, JumpTable, Tokenizer};

    fn parametrized_constructs_properly(code: &str) {
        let expected_tokens = Tokenizer::tokenize(code);
        let interpreter = Interpreter::new(code);

        assert_eq!(0, interpreter.program_counter);
        assert_eq!(0, interpreter.memory_pointer);
        assert_eq!(expected_tokens.clone(), interpreter.instructions);
        assert_eq!(JumpTable::from(&expected_tokens), interpreter.jump_table);
        assert_eq!([0u8; DEFAULT_TAPE_SIZE], interpreter.memory);
    }

    #[test]
    fn constructs_properly() {
        parametrized_constructs_properly("");
        parametrized_constructs_properly("+++>+++<---.");
        parametrized_constructs_properly("------");
        parametrized_constructs_properly("<<<<");
    }

    #[test]
    #[should_panic]
    fn panics_on_tape_out_of_range1() {
        Interpreter::new("").handle_move_left();
    }

    #[test]
    #[should_panic]
    fn panics_on_tape_out_of_range2() {
        let mut interpreter = Interpreter::new("");
        interpreter.memory_pointer = DEFAULT_TAPE_SIZE;
        interpreter.handle_move_right();
    }

    #[test]
    #[should_panic]
    fn panics_on_fetch_memory_out_of_range() {
        let code = "";
        let interpreter = Interpreter::new(code);
        interpreter.fetch_memory(DEFAULT_TAPE_SIZE);
    }

    #[test]
    #[should_panic]
    fn panics_on_fetch_memory_out_of_range_mut() {
        let code = "";
        let mut interpreter = Interpreter::new(code);
        interpreter.fetch_memory_mut(DEFAULT_TAPE_SIZE);
    }

    #[test]
    fn handle_move_right() {
        let mut interpreter = Interpreter::new("");
        interpreter.handle_move_right();

        assert_eq!(1, interpreter.memory_pointer);
    }

    #[test]
    fn handle_move_left() {
        let mut interpreter = Interpreter::new("");
        interpreter.memory_pointer = 1;
        interpreter.handle_move_left();
        assert_eq!(0, interpreter.memory_pointer);
    }

    #[test]
    fn handle_increment_no_wrap() {
        let mut interpreter = Interpreter::new("");
        interpreter.handle_increment();

        assert_eq!(1, *interpreter.fetch_memory(interpreter.memory_pointer));
    }

    #[test]
    fn handle_increment_wrap() {
        let mut interpreter = Interpreter::new("");
        *interpreter.fetch_memory_mut(interpreter.memory_pointer) = 255;
        interpreter.handle_increment();

        assert_eq!(0, *interpreter.fetch_memory(interpreter.memory_pointer));
    }

    #[test]
    fn handle_decrement_no_wrap() {
        let mut interpreter = Interpreter::new("");
        *interpreter.fetch_memory_mut(interpreter.memory_pointer) = 1;
        interpreter.handle_decrement();

        assert_eq!(0, *interpreter.fetch_memory(interpreter.memory_pointer));
    }

    #[test]
    fn handle_decrement_wrap() {
        let mut interpreter = Interpreter::new("");
        interpreter.handle_decrement();

        assert_eq!(255, *interpreter.fetch_memory(interpreter.memory_pointer));
    }
}
