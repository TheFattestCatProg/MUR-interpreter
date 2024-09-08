mod lexer;
mod meta;
mod meta2;
mod parser;
mod bignat;
mod vm;

use lexer::Lexer;
use parser::Parser;
use vm::execute;

fn main() {
    let lexer = Lexer::new("examples/3.mur").unwrap();
    let mut parser = Parser::new(lexer);
    match parser.parse() {
        Ok(code) => execute(code),
        Err(msg) => println!("{}", msg),
    }
}
