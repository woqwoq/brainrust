mod interpreter;
mod token;
mod tokenizer;

use interpreter::Interpreter;

fn main() {
    let code = "++++++[>++++++++++<-]>+++++.";
    let mut bf: Interpreter = Interpreter::new(code);

    bf.run(false);
}
