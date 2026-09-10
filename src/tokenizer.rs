use std::collections::HashMap;

use crate::token::Token;

pub struct JumpTable {}
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

pub struct Tokenizer {}
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

#[cfg(test)]
mod tokenizer_tests {
    use crate::tokenizer::{Token, Tokenizer};

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

    use crate::tokenizer::{JumpTable, Tokenizer};

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
