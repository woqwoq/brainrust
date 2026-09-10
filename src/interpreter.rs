use std::{
    fmt,
    io::{Read, Write},
};

use crate::program::Program;
use crate::token::Token;

const DEFAULT_TAPE_SIZE: usize = 500;

/// Stored at the current cell when `,` reads past the end of input.
const EOF_BYTE: u8 = 0;

pub struct Interpreter<I: Read, O: Write> {
    program_counter: usize,
    memory_pointer: usize,
    memory: [u8; DEFAULT_TAPE_SIZE],
    program: Program,
    input: I,
    output: O,
}

impl<I: Read, O: Write> Interpreter<I, O> {
    pub fn new(program_code: &str, input: I, output: O) -> Self {
        Interpreter {
            program_counter: 0,
            memory_pointer: 0,
            memory: [0u8; DEFAULT_TAPE_SIZE],
            program: Program::from(program_code),
            input,
            output,
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
        write!(self.output, "{memory_val_ascii}").unwrap();
        self.output.flush().unwrap();
    }

    pub fn handle_input(&mut self) {
        let byte = (&mut self.input)
            .bytes()
            .next()
            .transpose()
            .unwrap()
            .unwrap_or(EOF_BYTE);

        *self.fetch_memory_mut(self.memory_pointer) = byte;
    }

    fn jump_to_closest_bracket(&mut self) {
        self.program_counter = *self
            .program
            .get_matching_bracket(self.program_counter)
            .unwrap();
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
        self.program.fetch_instruction(self.program_counter)
    }

    pub fn fetch_memory(&self, index: usize) -> &u8 {
        if self.memory.len() <= index {
            panic!(
                "Trying to access memory at cell={} while DEFAULT_TAPE_SIZE={}",
                index, DEFAULT_TAPE_SIZE
            )
        }

        &self.memory[index]
    }

    pub fn fetch_memory_mut(&mut self, index: usize) -> &mut u8 {
        if self.memory.len() <= index {
            panic!(
                "Trying to access memory at cell={} while DEFAULT_TAPE_SIZE={}",
                index, DEFAULT_TAPE_SIZE
            )
        }

        &mut self.memory[index]
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
        while self.program.instructions.len() > self.program_counter {
            self.step();
        }
        if mem_dump {
            println!("{:?}", self.memory);
        }
    }
}

impl<I: Read, O: Write> fmt::Display for Interpreter<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instruction = match self.program.instructions.get(self.program_counter) {
            Some(token) => format!("{token:?}"),
            None => String::from("end of program"),
        };

        write!(
            f,
            "---\nPC: {}\nInstruction: {}\nMemory Pointer: {}\nMemory Dump: {:?}",
            self.program_counter, instruction, self.memory_pointer, self.memory
        )
    }
}

#[cfg(test)]
mod interpreter_tests {
    use std::io::{self, Cursor};

    use crate::tokenizer::{JumpTable, Tokenizer};

    use super::*;

    fn test_interpreter(code: &str) -> Interpreter<io::Empty, Vec<u8>> {
        Interpreter::new(code, io::empty(), Vec::new())
    }

    fn parametrized_constructs_properly(code: &str) {
        let expected_tokens = Tokenizer::tokenize(code);
        let interpreter = test_interpreter(code);

        assert_eq!(0, interpreter.program_counter);
        assert_eq!(0, interpreter.memory_pointer);
        assert_eq!(expected_tokens.clone(), interpreter.program.instructions);
        assert_eq!(
            JumpTable::from(&expected_tokens),
            interpreter.program.jump_table
        );
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
        test_interpreter("").handle_move_left();
    }

    #[test]
    #[should_panic]
    fn panics_on_tape_out_of_range2() {
        let mut interpreter = test_interpreter("");
        interpreter.memory_pointer = DEFAULT_TAPE_SIZE;
        interpreter.handle_move_right();
    }

    #[test]
    #[should_panic]
    fn panics_on_fetch_memory_out_of_range() {
        test_interpreter("").fetch_memory(DEFAULT_TAPE_SIZE);
    }

    #[test]
    #[should_panic]
    fn panics_on_fetch_memory_out_of_range_mut() {
        test_interpreter("").fetch_memory_mut(DEFAULT_TAPE_SIZE);
    }

    #[test]
    fn handle_move_right() {
        let mut interpreter = test_interpreter("");
        interpreter.handle_move_right();

        assert_eq!(1, interpreter.memory_pointer);
    }

    #[test]
    fn handle_move_left() {
        let mut interpreter = test_interpreter("");
        interpreter.memory_pointer = 1;
        interpreter.handle_move_left();
        assert_eq!(0, interpreter.memory_pointer);
    }

    #[test]
    fn handle_increment_no_wrap() {
        let mut interpreter = test_interpreter("");
        interpreter.handle_increment();

        assert_eq!(1, *interpreter.fetch_memory(interpreter.memory_pointer));
    }

    #[test]
    fn handle_increment_wrap() {
        let mut interpreter = test_interpreter("");
        *interpreter.fetch_memory_mut(interpreter.memory_pointer) = 255;
        interpreter.handle_increment();

        assert_eq!(0, *interpreter.fetch_memory(interpreter.memory_pointer));
    }

    #[test]
    fn handle_decrement_no_wrap() {
        let mut interpreter = test_interpreter("");
        *interpreter.fetch_memory_mut(interpreter.memory_pointer) = 1;
        interpreter.handle_decrement();

        assert_eq!(0, *interpreter.fetch_memory(interpreter.memory_pointer));
    }

    #[test]
    fn handle_decrement_wrap() {
        let mut interpreter = test_interpreter("");
        interpreter.handle_decrement();

        assert_eq!(255, *interpreter.fetch_memory(interpreter.memory_pointer));
    }

    #[test]
    fn handle_output_writes_cell_to_sink() {
        let mut interpreter = test_interpreter("");
        *interpreter.fetch_memory_mut(0) = b'A';

        interpreter.handle_output();

        assert_eq!(b"A", interpreter.output.as_slice());
    }

    #[test]
    fn handle_input_stores_single_bytes() {
        let mut interpreter = Interpreter::new("", Cursor::new(b"ab".to_vec()), Vec::new());

        interpreter.handle_input();
        assert_eq!(b'a', *interpreter.fetch_memory(0));

        interpreter.handle_input();
        assert_eq!(b'b', *interpreter.fetch_memory(0));
    }

    #[test]
    fn handle_input_stores_eof_byte_at_end_of_input() {
        let mut interpreter = test_interpreter("");

        interpreter.handle_input();

        assert_eq!(0, *interpreter.fetch_memory(0));
    }

    #[test]
    fn jump_to_closest_bracket1() {
        let mut interpreter = Interpreter::new("+[>++<-]", io::empty(), Vec::new());

        interpreter.step();
        assert_eq!(1, interpreter.program_counter);

        interpreter.jump_to_closest_bracket();
        assert_eq!(7, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_start_doesnt_jump_when_non_zero() {
        let mut interpreter = Interpreter::new("+[>+<-]", io::empty(), Vec::new());

        interpreter.step();

        interpreter.handle_loop_start();
        assert_eq!(1, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_start_jump_when_zero() {
        let mut interpreter = Interpreter::new("[>+<-]", io::empty(), Vec::new());

        interpreter.handle_loop_start();
        assert_eq!(5, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_close_doesnt_jump_when_lt_zero() {
        let mut interpreter = Interpreter::new("+[-]", io::empty(), Vec::new());
        interpreter.step();
        interpreter.step();
        interpreter.step();

        interpreter.handle_loop_close();
        assert_eq!(3, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_close_jump_when_gt_zero() {
        let mut interpreter = Interpreter::new("+[]", io::empty(), Vec::new());
        interpreter.step();
        interpreter.step();

        interpreter.handle_loop_close();
        assert_eq!(1, interpreter.program_counter);
    }

    #[test]
    #[should_panic]
    fn panics_when_stepping_past_program_end() {
        let mut interpreter = test_interpreter("+");
        interpreter.program_counter = 10;

        interpreter.step();
    }

    #[test]
    fn run_executes_loop_transfer_and_output() {
        let mut interpreter = test_interpreter("+++[>+<-]>.");

        interpreter.run(false);

        assert_eq!(0, *interpreter.fetch_memory(0));
        assert_eq!(3, *interpreter.fetch_memory(1));
        assert_eq!(b"\x03", interpreter.output.as_slice());
    }

    #[test]
    fn run_echoes_input_until_eof() {
        let mut interpreter = Interpreter::new(",[.,]", Cursor::new(b"hi".to_vec()), Vec::new());

        interpreter.run(false);

        assert_eq!(b"hi", interpreter.output.as_slice());
    }

    #[test]
    fn run_skips_comments() {
        let mut interpreter = test_interpreter("++comment.");

        interpreter.run(false);

        assert_eq!(b"\x02", interpreter.output.as_slice());
    }

    #[test]
    fn run_prints_memory_dump_when_requested() {
        let mut interpreter = test_interpreter("++");

        interpreter.run(true);

        assert_eq!(2, *interpreter.fetch_memory(0));
    }

    #[test]
    fn display_shows_state_and_survives_end_of_program() {
        let interpreter = test_interpreter("+");
        let text = format!("{interpreter}");
        assert!(text.contains("PC: 0"));
        assert!(text.contains("Instruction: Increment"));

        let mut interpreter = test_interpreter("+");
        interpreter.run(false);
        let text = format!("{interpreter}");
        assert!(text.contains("end of program"));
    }
}
