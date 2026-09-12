use std::collections::HashMap;
use std::fmt;

use crate::error::SyntaxError;
use crate::token::Token;
use crate::tokenizer::{JumpTable, Tokenizer};

#[derive(Clone)]
pub struct Program {
    pub instructions: Vec<Token>,
    pub jump_table: HashMap<usize, usize>,
}

impl Program {
    pub fn from(program_code: &str) -> Result<Self, SyntaxError> {
        let instructions = Tokenizer::tokenize(program_code);
        let jump_table = JumpTable::from(&instructions)?;
        Ok(Program {
            instructions,
            jump_table,
        })
    }

    pub fn has_instruction(&self, index: usize) -> bool {
        self.instructions.len() > index && self.instructions.get(index).is_some()
    }

    pub fn fetch_instruction(&self, index: usize) -> Option<Token> {
        self.instructions.get(index).cloned()
    }

    pub fn get_matching_bracket(&self, index: usize) -> Option<&usize> {
        self.jump_table.get(&index)
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut code = String::new();
        for instruction in &self.instructions {
            code.push(instruction.get_symbol());
        }

        write!(f, "{}", code)
    }
}

#[cfg(test)]
mod program_tests {
    use std::collections::HashMap;

    use crate::error::SyntaxError;
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

        let program = Program::from(code).unwrap();

        assert_eq!(expected_instructions.len(), program.instructions.len());
        assert_eq!(expected_instructions, program.instructions);
        assert_eq!(expected_jump_table, program.jump_table);
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

        let program = Program::from(code).unwrap();

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

        let program = Program::from(code).unwrap();

        for (x, y) in expected_jump_table.iter() {
            assert_eq!(*y, *program.get_matching_bracket(*x).unwrap());
        }
    }

    #[test]
    fn errors_on_malformed_program() {
        assert!(matches!(
            Program::from("["),
            Err(SyntaxError::UnmatchedLoopLeftBracket { position: 0, .. })
        ));
    }
}
