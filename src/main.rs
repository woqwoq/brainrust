use core::panic;
use std::{
    collections::HashMap,
    fmt,
    io::{self, Write},
};

const DEFAULT_TAPE_SIZE: usize = 15;

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

struct JumpTable {}
impl JumpTable {
    pub fn from(tokens: &[Token]) -> HashMap<usize, usize> {
        let mut stack: Vec<usize> = Vec::new();
        let mut jump_table = HashMap::new();

        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::LoopStart => stack.push(i),
                Token::LoopClose => {
                    if let Some(n) = stack.last() {
                        jump_table.insert(*n, i);
                        jump_table.insert(i, *n);
                        stack.pop();
                    } else {
                        panic!("Unmatched ']' at position={}", i)
                    }
                }
                _ => {}
            }
        }
        if !stack.is_empty() {
            panic!("Unmatched '[' at positions={:?}", stack);
        }

        jump_table
    }
}

struct Tokenizer {}
impl Tokenizer {
    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        let mut comment_buffer = String::new();
        let mut dump_comment;

        for char in input.chars() {
            dump_comment = true;
            let current_token = match char {
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

            if let Some(t) = current_token {
                if dump_comment && !comment_buffer.is_empty() {
                    tokens.push(Token::Comment(comment_buffer.clone()));
                    comment_buffer.clear();
                }
                tokens.push(t);
            }
        }
        if !comment_buffer.is_empty() {
            tokens.push(Token::Comment(comment_buffer.clone()));
            comment_buffer.clear();
        }

        tokens
    }
}

struct Interpreter {
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
        let memory_value = self.fetch_memory_mut(self.memory_pointer);

        if *memory_value == 0 {
            self.jump_to_closest_bracket();
        }
    }

    pub fn handle_loop_close(&mut self) {
        let memory_value = self.fetch_memory_mut(self.memory_pointer);

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

    pub fn run(&mut self) {
        while self.instructions.len() > self.program_counter {
            self.step();
            // println!("{}", self);
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
    let code = String::from(
        "
,                           ;read character and store it in p1
------------------------------------------------   ;return ascii to Dec
>                           ;move pointer to p2 (second byte)
,                           ;read character and store it in p2
------------------------------------------------ ;return ascii to Dec
[                           ; enter loop
-                           ; decrement p2
<                           ; move to p1
+                           ; increment p1
>                           ; move to p2
]                           ; we exit the loop when the last cell is empty
<                           ;go back to p1
++++++++++++++++++++++++++++++++++++++++++++++++     ;return Dec to ascii
.                           ;print p1
",
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

    #[test]
    fn trailing_comment_tokenized_properly1() {
        let code = "+>comment+<+comment";
        let expected = vec![
            Token::Increment,
            Token::MoveRight,
            Token::Comment(String::from("comment")),
            Token::Increment,
            Token::MoveLeft,
            Token::Increment,
            Token::Comment(String::from("comment")),
        ];

        assert_eq!(expected, Tokenizer::tokenize(code));
    }
    #[test]
    fn trailing_comment_tokenized_properly2() {
        let code = "comment";
        let expected = vec![Token::Comment(String::from("comment"))];

        assert_eq!(expected, Tokenizer::tokenize(code));
    }
}

#[cfg(test)]
mod jump_table_tests {
    use std::collections::HashMap;

    use crate::{JumpTable, Tokenizer};

    #[test]
    fn well_formed_jump_table() {
        let code = "[[[+++]]]";
        let mut expected: HashMap<usize, usize> = HashMap::new();
        expected.insert(0, 8);
        expected.insert(1, 7);
        expected.insert(2, 6);
        expected.insert(8, 0);
        expected.insert(7, 1);
        expected.insert(6, 2);

        assert_eq!(expected, JumpTable::from(&Tokenizer::tokenize(code)));
    }

    #[test]
    #[should_panic]
    fn malformed_jump_table_panics1() {
        let code = "[";
        JumpTable::from(&Tokenizer::tokenize(code));
    }

    #[test]
    #[should_panic]
    fn malformed_jump_table_panics2() {
        let code = "]";
        JumpTable::from(&Tokenizer::tokenize(code));
    }
}

#[cfg(test)]
mod interpreter_tests {
    use crate::{DEFAULT_TAPE_SIZE, Interpreter, JumpTable, Tokenizer};

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
        let code = "<";
        Interpreter::new(code).run();
    }

    #[test]
    #[should_panic]
    fn panics_on_tape_out_of_range2() {
        let code = ">";
        let mut interpreter = Interpreter::new(code);
        interpreter.memory_pointer = DEFAULT_TAPE_SIZE;
        interpreter.run();
    }

    #[test]
    #[should_panic]
    fn panics_on_fetch_memory_out_of_range() {
        let code = "";
        let interpreter = Interpreter::new(code);
        interpreter.fetch_memory(DEFAULT_TAPE_SIZE + 1);
    }

    #[test]
    #[should_panic]
    fn panics_on_fetch_memory_out_of_range_mut() {
        let code = "";
        let mut interpreter = Interpreter::new(code);
        interpreter.fetch_memory_mut(DEFAULT_TAPE_SIZE + 1);
    }
}
