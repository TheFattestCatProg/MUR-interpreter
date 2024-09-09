mod cli_parser;
mod lexer;
mod meta;
mod meta2;
mod parser;
mod bignat;
mod vm;

use std::{fs::File, path::Path};
use cli_parser::CmdArgs;
use lexer::Lexer;
use parser::Parser;
use vm::execute;

fn main() {
    let args = match CmdArgs::parse(std::env::args()) {
        Ok(t) => t,
        Err(msg) => {
            println!("{}", msg);
            return;
        }
    };

    let path = Path::new(args.filepath());

    if !path.exists() {
        println!("File doesn't exist");
        return;
    }

    if !path.is_file() {
        println!("Specified path is not file");
        return;
    }

    let Ok(file) = File::open(args.filepath()) else {
        println!("Cannot open file");
        return;
    };

    let lexer = Lexer::new(file);
    let mut parser = Parser::new(lexer);

    if args.only_expand() {
        if let Err(msg) = parser.print_debug() {
            println!("{}", msg);
        }
    }
    else {
        match parser.parse() {
            Ok(code) => execute(code),
            Err(msg) => {
                println!("{}", msg);
                return;
            },
        }
    }
}
