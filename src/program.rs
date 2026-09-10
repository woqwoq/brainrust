use std::collections::HashMap;

use crate::token::Token;
use crate::tokenizer::{JumpTable, Tokenizer};

pub struct Program {
    pub instructions: Vec<Token>,
    pub jump_table: HashMap<usize, usize>,
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
}
