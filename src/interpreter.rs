use std::{
    fmt,
    io::{Read, Write},
};

use crate::program::Program;
use crate::token::Token;

const DEFAULT_TAPE_SIZE: usize = 500;

/// Stored at the current cell when `,` reads past the end of input.
const EOF_BYTE: u8 = 0;

#[derive(Debug)]
pub enum RuntimeError {
    /// `<` at cell 0 or `>` past the last cell.
    MemoryPointerOutOfBounds { pc: usize, pointer: usize },

    /// Memory access outside the tape.
    MemoryAccessOutOfBounds { cell: usize, tape_size: usize },

    /// Program counter outside the program.
    InstructionFetch { pc: usize },

    /// I/O failure on the input or output stream.
    Io(std::io::Error),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryPointerOutOfBounds { pc, pointer } => {
                write!(f, "memory pointer {pointer} out of bounds at pc={pc}")
            }
            Self::MemoryAccessOutOfBounds { cell, tape_size } => {
                write!(
                    f,
                    "memory access at cell={cell} while tape size is {tape_size}"
                )
            }
            Self::InstructionFetch { pc } => {
                write!(f, "failed to fetch instruction at pc={pc}")
            }
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

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

    pub fn handle_move_left(&mut self) -> Result<(), RuntimeError> {
        if self.memory_pointer == 0 {
            return Err(RuntimeError::MemoryPointerOutOfBounds {
                pc: self.program_counter,
                pointer: self.memory_pointer,
            });
        }

        self.memory_pointer -= 1;

        Ok(())
    }

    pub fn handle_move_right(&mut self) -> Result<(), RuntimeError> {
        if self.memory_pointer + 1 >= DEFAULT_TAPE_SIZE {
            return Err(RuntimeError::MemoryPointerOutOfBounds {
                pc: self.program_counter,
                pointer: self.memory_pointer,
            });
        }

        self.memory_pointer += 1;

        Ok(())
    }

    pub fn handle_increment(&mut self) -> Result<(), RuntimeError> {
        let memory_value = self.fetch_memory_mut(self.memory_pointer)?;
        *memory_value = memory_value.wrapping_add(1);

        Ok(())
    }

    pub fn handle_decrement(&mut self) -> Result<(), RuntimeError> {
        let memory_value = self.fetch_memory_mut(self.memory_pointer)?;
        *memory_value = memory_value.wrapping_sub(1);

        Ok(())
    }

    pub fn handle_output(&mut self) -> Result<(), RuntimeError> {
        let memory_val_ascii =
            char::from_u32(*self.fetch_memory(self.memory_pointer)? as u32).unwrap();
        write!(self.output, "{memory_val_ascii}").map_err(RuntimeError::Io)?;
        self.output.flush().map_err(RuntimeError::Io)?;

        Ok(())
    }

    pub fn handle_input(&mut self) -> Result<(), RuntimeError> {
        let byte = (&mut self.input)
            .bytes()
            .next()
            .transpose()
            .map_err(RuntimeError::Io)?
            .unwrap_or(EOF_BYTE);

        *self.fetch_memory_mut(self.memory_pointer)? = byte;

        Ok(())
    }

    fn jump_to_closest_bracket(&mut self) {
        self.program_counter = *self
            .program
            .get_matching_bracket(self.program_counter)
            .unwrap();
    }

    pub fn handle_loop_start(&mut self) -> Result<(), RuntimeError> {
        let memory_value = self.fetch_memory(self.memory_pointer)?;

        if *memory_value == 0 {
            self.jump_to_closest_bracket();
        }

        Ok(())
    }

    pub fn handle_loop_close(&mut self) -> Result<(), RuntimeError> {
        let memory_value = self.fetch_memory(self.memory_pointer)?;

        if *memory_value > 0 {
            self.jump_to_closest_bracket();
        }

        Ok(())
    }

    pub fn fetch_instruction(&self) -> Option<Token> {
        self.program.fetch_instruction(self.program_counter)
    }

    pub fn fetch_memory(&self, index: usize) -> Result<&u8, RuntimeError> {
        if self.memory.len() <= index {
            return Err(RuntimeError::MemoryAccessOutOfBounds {
                cell: index,
                tape_size: self.memory.len(),
            });
        }

        Ok(&self.memory[index])
    }

    pub fn fetch_memory_mut(&mut self, index: usize) -> Result<&mut u8, RuntimeError> {
        if self.memory.len() <= index {
            return Err(RuntimeError::MemoryAccessOutOfBounds {
                cell: index,
                tape_size: self.memory.len(),
            });
        }

        Ok(&mut self.memory[index])
    }

    pub fn execute_instruction(&mut self, instruction: Token) -> Result<(), RuntimeError> {
        match instruction {
            Token::MoveLeft => self.handle_move_left(),
            Token::MoveRight => self.handle_move_right(),
            Token::Increment => self.handle_increment(),
            Token::Decrement => self.handle_decrement(),
            Token::Output => self.handle_output(),
            Token::Input => self.handle_input(),
            Token::LoopStart => self.handle_loop_start(),
            Token::LoopClose => self.handle_loop_close(),
            Token::Comment(_) => Ok(()),
        }
    }

    pub fn step(&mut self) -> Result<(), RuntimeError> {
        if let Some(instruction) = self.fetch_instruction() {
            self.execute_instruction(instruction)?;
            self.program_counter += 1;
            Ok(())
        } else {
            Err(RuntimeError::InstructionFetch {
                pc: self.program_counter,
            })
        }
    }

    pub fn run(&mut self, mem_dump: bool) -> Result<(), RuntimeError> {
        while self.program.get_instruction_count() > self.program_counter {
            self.step()?;
        }
        if mem_dump {
            println!("{:?}", self.memory);
        }

        Ok(())
    }
}

impl<I: Read, O: Write> fmt::Display for Interpreter<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instruction = match self.program.fetch_instruction(self.program_counter) {
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

    use super::*;

    fn test_interpreter(code: &str) -> Interpreter<io::Empty, Vec<u8>> {
        Interpreter::new(code, io::empty(), Vec::new())
    }

    fn parametrized_constructs_properly(code: &str) {
        let interpreter = test_interpreter(code);

        assert_eq!(0, interpreter.program_counter);
        assert_eq!(0, interpreter.memory_pointer);
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
    fn returns_error_on_tape_underflow() {
        let mut interpreter = test_interpreter("");

        assert!(matches!(
            interpreter.handle_move_left(),
            Err(RuntimeError::MemoryPointerOutOfBounds { .. })
        ));
    }

    #[test]
    fn returns_error_on_tape_overflow() {
        let mut interpreter = test_interpreter("");
        interpreter.memory_pointer = DEFAULT_TAPE_SIZE;

        assert!(matches!(
            interpreter.handle_move_right(),
            Err(RuntimeError::MemoryPointerOutOfBounds { .. })
        ));
    }

    #[test]
    fn returns_error_on_memory_access_out_of_range() {
        let interpreter = test_interpreter("");

        assert!(matches!(
            interpreter.fetch_memory(DEFAULT_TAPE_SIZE),
            Err(RuntimeError::MemoryAccessOutOfBounds { .. })
        ));
    }

    #[test]
    fn returns_error_on_memory_access_out_of_range_mut() {
        let mut interpreter = test_interpreter("");

        assert!(matches!(
            interpreter.fetch_memory_mut(DEFAULT_TAPE_SIZE),
            Err(RuntimeError::MemoryAccessOutOfBounds { .. })
        ));
    }

    #[test]
    fn handle_move_right() {
        let mut interpreter = test_interpreter("");
        interpreter.handle_move_right().unwrap();

        assert_eq!(1, interpreter.memory_pointer);
    }

    #[test]
    fn handle_move_left() {
        let mut interpreter = test_interpreter("");
        interpreter.memory_pointer = 1;
        interpreter.handle_move_left().unwrap();
        assert_eq!(0, interpreter.memory_pointer);
    }

    #[test]
    fn handle_increment_no_wrap() {
        let mut interpreter = test_interpreter("");
        interpreter.handle_increment().unwrap();

        assert_eq!(
            1,
            *interpreter
                .fetch_memory(interpreter.memory_pointer)
                .unwrap()
        );
    }

    #[test]
    fn handle_increment_wrap() {
        let mut interpreter = test_interpreter("");
        *interpreter
            .fetch_memory_mut(interpreter.memory_pointer)
            .unwrap() = 255;
        interpreter.handle_increment().unwrap();

        assert_eq!(
            0,
            *interpreter
                .fetch_memory(interpreter.memory_pointer)
                .unwrap()
        );
    }

    #[test]
    fn handle_decrement_no_wrap() {
        let mut interpreter = test_interpreter("");
        *interpreter
            .fetch_memory_mut(interpreter.memory_pointer)
            .unwrap() = 1;
        interpreter.handle_decrement().unwrap();

        assert_eq!(
            0,
            *interpreter
                .fetch_memory(interpreter.memory_pointer)
                .unwrap()
        );
    }

    #[test]
    fn handle_decrement_wrap() {
        let mut interpreter = test_interpreter("");
        interpreter.handle_decrement().unwrap();

        assert_eq!(
            255,
            *interpreter
                .fetch_memory(interpreter.memory_pointer)
                .unwrap()
        );
    }

    #[test]
    fn handle_output_writes_cell_to_sink() {
        let mut interpreter = test_interpreter("");
        *interpreter.fetch_memory_mut(0).unwrap() = b'A';

        interpreter.handle_output().unwrap();

        assert_eq!(b"A", interpreter.output.as_slice());
    }

    #[test]
    fn handle_input_stores_single_bytes() {
        let mut interpreter = Interpreter::new("", Cursor::new(b"ab".to_vec()), Vec::new());

        interpreter.handle_input().unwrap();
        assert_eq!(b'a', *interpreter.fetch_memory(0).unwrap());

        interpreter.handle_input().unwrap();
        assert_eq!(b'b', *interpreter.fetch_memory(0).unwrap());
    }

    #[test]
    fn handle_input_stores_eof_byte_at_end_of_input() {
        let mut interpreter = test_interpreter("");

        interpreter.handle_input().unwrap();

        assert_eq!(0, *interpreter.fetch_memory(0).unwrap());
    }

    #[test]
    fn jump_to_closest_bracket1() {
        let mut interpreter = Interpreter::new("+[>++<-]", io::empty(), Vec::new());

        interpreter.step().unwrap();
        assert_eq!(1, interpreter.program_counter);

        interpreter.jump_to_closest_bracket();
        assert_eq!(7, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_start_doesnt_jump_when_non_zero() {
        let mut interpreter = Interpreter::new("+[>+<-]", io::empty(), Vec::new());

        interpreter.step().unwrap();

        interpreter.handle_loop_start().unwrap();
        assert_eq!(1, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_start_jump_when_zero() {
        let mut interpreter = Interpreter::new("[>+<-]", io::empty(), Vec::new());

        interpreter.handle_loop_start().unwrap();
        assert_eq!(5, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_close_doesnt_jump_when_lt_zero() {
        let mut interpreter = Interpreter::new("+[-]", io::empty(), Vec::new());
        interpreter.step().unwrap();
        interpreter.step().unwrap();
        interpreter.step().unwrap();

        interpreter.handle_loop_close().unwrap();
        assert_eq!(3, interpreter.program_counter);
    }

    #[test]
    fn handle_loop_close_jump_when_gt_zero() {
        let mut interpreter = Interpreter::new("+[]", io::empty(), Vec::new());
        interpreter.step().unwrap();
        interpreter.step().unwrap();

        interpreter.handle_loop_close().unwrap();
        assert_eq!(1, interpreter.program_counter);
    }

    #[test]
    fn returns_error_when_stepping_past_program_end() {
        let mut interpreter = test_interpreter("+");
        interpreter.program_counter = 10;

        assert!(matches!(
            interpreter.step(),
            Err(RuntimeError::InstructionFetch { .. })
        ));
    }

    #[test]
    fn run_stops_with_error_on_pointer_underflow() {
        let mut interpreter = test_interpreter("<<<");

        assert!(matches!(
            interpreter.run(false),
            Err(RuntimeError::MemoryPointerOutOfBounds { .. })
        ));
    }

    #[test]
    fn run_executes_loop_transfer_and_output() {
        let mut interpreter = test_interpreter("+++[>+<-]>.");

        interpreter.run(false).unwrap();

        assert_eq!(0, *interpreter.fetch_memory(0).unwrap());
        assert_eq!(3, *interpreter.fetch_memory(1).unwrap());
        assert_eq!(b"\x03", interpreter.output.as_slice());
    }

    #[test]
    fn run_echoes_input_until_eof() {
        let mut interpreter = Interpreter::new(",[.,]", Cursor::new(b"hi".to_vec()), Vec::new());

        interpreter.run(false).unwrap();

        assert_eq!(b"hi", interpreter.output.as_slice());
    }

    #[test]
    fn run_skips_comments() {
        let mut interpreter = test_interpreter("++comment.");

        interpreter.run(false).unwrap();

        assert_eq!(b"\x02", interpreter.output.as_slice());
    }

    #[test]
    fn run_prints_memory_dump_when_requested() {
        let mut interpreter = test_interpreter("++");

        interpreter.run(true).unwrap();

        assert_eq!(2, *interpreter.fetch_memory(0).unwrap());
    }

    #[test]
    fn display_shows_state_and_survives_end_of_program() {
        let interpreter = test_interpreter("+");
        let text = format!("{interpreter}");
        assert!(text.contains("PC: 0"));
        assert!(text.contains("Instruction: Increment"));

        let mut interpreter = test_interpreter("+");
        interpreter.run(false).unwrap();
        let text = format!("{interpreter}");
        assert!(text.contains("end of program"));
    }
}

#[cfg(test)]
mod runtime_error_test {
    use std::{error::Error, io};

    use crate::interpreter::RuntimeError;

    #[test]
    fn source_returns_some() {
        let io_error = io::Error::new(io::ErrorKind::ResourceBusy, "");
        let runtime_error = RuntimeError::Io(io::Error::new(io::ErrorKind::ResourceBusy, ""));

        assert_eq!(
            io_error.to_string(),
            runtime_error.source().unwrap().to_string()
        );
    }

    #[test]
    fn source_returns_none() {
        let runtime_error = RuntimeError::InstructionFetch { pc: 0 };

        assert!(runtime_error.source().is_none());
    }

    #[test]
    fn fmt_instruction_fetch_err() {
        let runtime_error = RuntimeError::InstructionFetch { pc: 0 };

        assert_eq!(
            "failed to fetch instruction at pc=0",
            format!("{}", runtime_error)
        );
    }

    #[test]
    fn fmt_memory_pointer_out_of_bounds_err() {
        let runtime_error = RuntimeError::MemoryPointerOutOfBounds { pc: 0, pointer: 0 };

        assert_eq!(
            "memory pointer 0 out of bounds at pc=0",
            format!("{}", runtime_error)
        );
    }

    #[test]
    fn fmt_memory_access_out_of_bounds_err() {
        let runtime_error = RuntimeError::MemoryAccessOutOfBounds {
            cell: 0,
            tape_size: 30000,
        };

        assert_eq!(
            "memory access at cell=0 while tape size is 30000",
            format!("{}", runtime_error)
        );
    }

    #[test]
    fn fmt_io_err() {
        let runtime_error = RuntimeError::Io(io::Error::new(
            io::ErrorKind::ResourceBusy,
            "super serious error",
        ));

        assert_eq!(
            "io error: super serious error",
            format!("{}", runtime_error)
        );
    }
}
