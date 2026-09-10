mod interpreter;
mod program;
mod token;
mod tokenizer;

use std::io;

use interpreter::Interpreter;

fn main() {
    let code = "++++++[>++++++++++<-]>+++++.";
    let mut bf = Interpreter::new(code, io::stdin(), io::stdout());

    bf.run(false);
}
