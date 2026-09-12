#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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

impl Token {
    pub fn get_symbol(&self) -> char {
        match self {
            Token::MoveLeft => '<',
            Token::MoveRight => '>',
            Token::Increment => '+',
            Token::Decrement => '-',
            Token::Output => '.',
            Token::Input => ',',
            Token::LoopStart => '[',
            Token::LoopClose => ']',
            Token::Comment(_) => 'c',
        }
    }
}
