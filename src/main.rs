use std::{
    collections::HashSet,
    fmt,
    io::{self, Write},
};

#[derive(Debug, PartialEq, Clone)]
enum Token {
    MoveLeft,
    MoveRight,
    Increment,
    Decrement,
    Output,
    Input,
    LoopStart,
    LoopClose,
    Comment(String),
}

struct Tokenizer {}
impl Tokenizer {
    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let chars = input.chars();

        let mut comment_buffer = String::new();
        let mut dump_comment;

        let chars_iter = chars.into_iter();
        for char in chars_iter {
            dump_comment = true;
            let current_tokent = match char {
                '<' => Some(Token::MoveLeft),
                '>' => Some(Token::MoveRight),
                '+' => Some(Token::Increment),
                '-' => Some(Token::Decrement),
                '.' => Some(Token::Output),
                ',' => Some(Token::Input),
                '[' => Some(Token::LoopStart),
                ']' => Some(Token::LoopClose),
                c => {
                    comment_buffer.push(c);
                    dump_comment = false;
                    None
                }
            };

            if let Some(t) = current_tokent {
                if dump_comment && !comment_buffer.is_empty() {
                    tokens.push(Token::Comment(comment_buffer.clone()));
                    comment_buffer.clear();
                }
                tokens.push(t);
            }
        }

        tokens
    }
}

const DEFAULT_TAPE_SIZE: usize = 15;

struct Interpreter {
    program_counter: usize,
    instructions: Vec<Token>,
    memory_pointer: usize,
    memory: [u8; DEFAULT_TAPE_SIZE],

    loop_starts: HashSet<usize>,
}

impl Interpreter {
    pub fn new(instructions: &str) -> Self {
        Interpreter {
            program_counter: 0,
            instructions: Tokenizer::tokenize(instructions),
            memory_pointer: 0,
            memory: [0u8; DEFAULT_TAPE_SIZE],
            loop_starts: HashSet::new(),
        }
    }

    pub fn handle_move_left(&mut self) {
        let (res, ovf) = self.memory_pointer.overflowing_sub(1);
        if ovf {
            panic!(
                "Memory tape pointer left allowed bounds on pc={}",
                self.program_counter
            )
        }

        self.memory_pointer = res;

        self.program_counter += 1;
    }

    pub fn handle_move_right(&mut self) {
        let (res, ovf) = self.memory_pointer.overflowing_add(1);
        if ovf {
            panic!(
                "Memory tape pointer left allowed bounds on pc={}",
                self.program_counter
            )
        }

        self.memory_pointer = res;

        self.program_counter += 1;
    }

    pub fn handle_increment(&mut self) {
        let memory_value = self.fetch_memory_mut(self.memory_pointer);
        *memory_value += 1;

        self.program_counter += 1;
    }

    pub fn handle_decrement(&mut self) {
        let memory_value = self.fetch_memory_mut(self.memory_pointer);
        *memory_value -= 1;

        self.program_counter += 1;
    }

    pub fn handle_output(&mut self) {
        let memory_val_ascii =
            char::from_u32(*self.fetch_memory(self.memory_pointer) as u32).unwrap();
        print!("{}", memory_val_ascii);
        io::stdout().flush().unwrap();

        self.program_counter += 1;
    }

    pub fn handle_input(&mut self) {
        let mut string_buffer = String::new();

        io::stdin().read_line(&mut string_buffer).unwrap();

        if let Ok(n) = string_buffer.trim().parse::<u8>() {
            *self.fetch_memory_mut(self.memory_pointer) = n;
        }

        self.program_counter += 1;
    }

    fn jump_to_closest_loop_close(&mut self) {
        while self.fetch_instruction().unwrap() != Token::LoopClose {
            self.program_counter += 1;
        }
    }

    pub fn handle_loop_start(&mut self) {
        let memory_value = self.fetch_memory_mut(self.memory_pointer);

        if *memory_value == 0 {
            self.jump_to_closest_loop_close();
        }

        self.program_counter += 1;
    }

    fn jump_to_closest_loop_start(&mut self) {
        while self.fetch_instruction().unwrap() != Token::LoopStart {
            self.program_counter -= 1;
        }
    }

    pub fn handle_loop_close(&mut self) {
        let memory_value = self.fetch_memory_mut(self.memory_pointer);

        if *memory_value > 0 {
            self.jump_to_closest_loop_start();
        }

        self.program_counter += 1;
    }

    pub fn fetch_instruction(&self) -> Option<Token> {
        self.instructions.get(self.program_counter).cloned()
    }

    pub fn fetch_memory(&self, index: usize) -> &u8 {
        if self.memory.len() < index {
            panic!(
                "Trying to access memory at cell={} while DEFAULT_TAPE_SIZE={}",
                index, DEFAULT_TAPE_SIZE
            )
        }

        if let Some(memory_value) = self.memory.get(self.memory_pointer) {
            memory_value
        } else {
            panic!(
                "Cannot acces value at tape position={} during executing instruction at pc={}",
                self.memory_pointer, self.program_counter
            )
        }
    }

    pub fn fetch_memory_mut(&mut self, index: usize) -> &mut u8 {
        if self.memory.len() < index {
            panic!(
                "Trying to access memory at cell={} while DEFAULT_TAPE_SIZE={}",
                index, DEFAULT_TAPE_SIZE
            )
        }

        if let Some(memory_value) = self.memory.get_mut(self.memory_pointer) {
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
        } else {
            panic!("Failed to fetch instruction at pc={}", self.program_counter) // not expected
        }
    }

    pub fn run(&mut self) {
        while self.instructions.len() > self.program_counter {
            self.step();
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
fn main() {
    let mut code = String::from(
        "++++++++++[>+>+++>+++++++>++++++++++<<<<-]>>>>+++++++++++.-------.<<++.>>+++++++++++.-----------.+.+++++++++++.<<.>>-------.------------.+++++++++++++.<<.>>++++++.------------.+.++++++++++.<<.>>----------.++++++++++.<<.>>------------.++++++++..-----------.",
    );
    let mut bf: Interpreter = Interpreter::new(&code);

    bf.run();
}

#[cfg(test)]
mod tokenizer_tests {
    use crate::{Token, Tokenizer};

    #[test]
    fn tokenizes_properly() {
        let code = "<>+-.,[comment]c[c]";
        let expected = vec![
            Token::MoveLeft,
            Token::MoveRight,
            Token::Increment,
            Token::Decrement,
            Token::Output,
            Token::Input,
            Token::LoopStart,
            Token::Comment(String::from("comment")),
            Token::LoopClose,
            Token::Comment(String::from("c")),
            Token::LoopStart,
            Token::Comment(String::from("c")),
            Token::LoopClose,
        ];

        assert_eq!(expected, Tokenizer::tokenize(code));
    }
}
