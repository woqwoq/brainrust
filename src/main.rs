use std::fmt;

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

struct Interpreter {
    pc: u8,
    instructions: Vec<Token>,
    tape: [u8; 256],
    memory_pointer: u8,
}

impl Interpreter {
    pub fn new(instructions: &str) -> Self {
        Interpreter {
            pc: 0u8,
            instructions: Tokenizer::tokenize(instructions),
            memory_pointer: 0u8,
            tape: [0u8; 256],
        }
    }

    pub fn handle_move_left(&mut self) {
        let (res, ovf) = self.memory_pointer.overflowing_sub(1);
        if ovf {
            panic!("Memory tape pointer left allowed bounds on pc={}", self.pc)
        }

        self.memory_pointer = res
    }

    pub fn handle_move_right(&mut self) {
        self.memory_pointer += 1;
    }

    pub fn fetch_instruction(&self) -> Option<Token> {
        self.instructions.get(self.pc as usize).cloned()
    }

    pub fn execute_instruction(&mut self, instruction: Token) {
        match instruction {
            Token::MoveLeft => self.handle_move_left(),
            Token::MoveRight => self.handle_move_right(),
            _ => todo!("Only 2 instructions currently supported"),
        }
    }

    pub fn step(&mut self) {
        if let Some(instruction) = self.fetch_instruction() {
            self.execute_instruction(instruction);
            self.pc += 1;
        } else {
            panic!("Failed to fetch instruction at pc={}", self.pc) // not expected
        }
    }

    pub fn run(&mut self) {
        while self.instructions.len() > (self.pc as usize) {
            self.step();
        }
    }
}

impl fmt::Display for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PC: {}\nMemory Pointer: {}",
            self.pc, self.memory_pointer
        )
    }
}
fn main() {
    let code = ">>><<<<";
    let mut bf: Interpreter = Interpreter::new(code);

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
