mod lexer;
mod parser;
mod bignat;
mod vm;

use lexer::Lexer;
use parser::Parser;
use vm::execute;



fn main() {
    let lexer = Lexer::new("examples/1.mur").unwrap();
    let mut parser = Parser::new(lexer);

    let code = parser.parse().unwrap();

    execute(code);
}
