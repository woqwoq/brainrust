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

fn main() {
    todo!();
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
