use std::collections::HashMap;

use crate::token::Token;
use crate::tokenizer::{JumpTable, Tokenizer};

pub struct Program {
    instructions: Vec<Token>,
    jump_table: HashMap<usize, usize>,
}

impl Program {
    pub fn from(program_code: &str) -> Self {
        let instructions = Tokenizer::tokenize(program_code);
        let jump_table = JumpTable::from(&instructions);
        Program {
            instructions,
            jump_table,
        }
    }

    pub fn fetch_instruction(&self, index: usize) -> Option<Token> {
        self.instructions.get(index).cloned()
    }

    pub fn get_matching_bracket(&self, index: usize) -> Option<&usize> {
        self.jump_table.get(&index)
    }

    pub fn get_instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn get_all_instructions(&self) -> Vec<Token> {
        self.instructions.clone()
    }

    pub fn get_jump_table(&self) -> HashMap<usize, usize> {
        self.jump_table.clone()
    }
}

#[cfg(test)]
mod program_tests {
    use std::collections::HashMap;

    use crate::{program::Program, token::Token};

    #[test]
    fn program_constructs_properly() {
        let code = "<>+-.,[comment]c[c]";
        let expected_instructions = vec![
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
        let mut expected_jump_table = HashMap::new();
        expected_jump_table.insert(6, 8);
        expected_jump_table.insert(8, 6);
        expected_jump_table.insert(10, 12);
        expected_jump_table.insert(12, 10);

        let program = Program::from(code);

        assert_eq!(expected_instructions.len(), program.get_instruction_count());
        assert_eq!(expected_instructions, program.get_all_instructions());
        assert_eq!(expected_jump_table, program.get_jump_table());
    }

    #[test]
    fn fetch_instruction() {
        let code = "<>+-.,[comment]c[c]";
        let expected_instructions = vec![
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

        let program = Program::from(code);

        for (i, instruction) in expected_instructions.into_iter().enumerate() {
            assert_eq!(instruction, program.fetch_instruction(i).unwrap());
        }
    }

    #[test]
    fn get_matching_bracket() {
        let code = "<>+-.,[comment]c[c]";

        let mut expected_jump_table = HashMap::new();
        expected_jump_table.insert(6, 8);
        expected_jump_table.insert(8, 6);
        expected_jump_table.insert(10, 12);
        expected_jump_table.insert(12, 10);

        let program = Program::from(code);

        for (x, y) in expected_jump_table.iter() {
            assert_eq!(*y, *program.get_matching_bracket(*x).unwrap());
        }
    }
}
