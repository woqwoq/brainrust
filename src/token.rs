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
