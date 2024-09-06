use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::LinkedList;
use std::rc::Rc;

type PosType = u32;

#[derive(Clone, Copy)]
pub struct Position {
    pub line: PosType,
    pub word: PosType,
}

impl Position {
    pub fn new(line: PosType, word: PosType) -> Self {
        Position {
            line: line,
            word: word
        }
    }

    pub fn str(&self) -> String {
        return format!("{}:{}", self.line, self.word);
    }
}

pub enum Token {
    None,
    Eof,
    NewLine(PosType), // line
    Register(Rc<String>, Position),
    Label(Rc<String>, Position),
    Op(Rc<String>, Position)
}

impl Token {
    pub fn pos_str(&self) -> String {
        match self {
            Token::None => String::from("?:?"),
            Token::Eof => String::from("-1:-1"),

            Token::NewLine(line) => format!("{}:-1", line),

            Token::Register(_, pos)
            | Token::Label(_, pos)
            | Token::Op(_, pos)  => format!("{}:{}", pos.line, pos.word),
        }
    }

    pub fn str(&self) -> String {
        match self {
            Token::None       => String::from("None"),
            Token::Eof        => String::from("EOF"),
            Token::NewLine(_) => String::from("NewLine"),

            Token::Register(name, _) => format!("Reg(%{})", name),
            Token::Label(name, _)    => format!("Lab(@{})", name),
            Token::Op(name, _)       => format!("Op({})", name),
        }
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Eof => Self::Eof,
            Self::NewLine(line) => Self::NewLine(*line),
            Self::Register(name, pos) => Self::Register(Rc::clone(name), *pos),
            Self::Label(name, pos)    => Self::Label(Rc::clone(name), *pos),
            Self::Op(name, pos)       => Self::Op(Rc::clone(name), *pos),
        }
    }
}

pub struct Lexer {
    file_reader: BufReader<File>,
    token_buffer: LinkedList<Token>, // read whole line and place to buffer
    line: PosType,

    curr_token: Token,
}

impl Lexer {
    pub fn new(path: &str) -> io::Result<Self> {
        Ok(Lexer {
            file_reader: BufReader::new(File::open(path)?),
            token_buffer: LinkedList::new(),
            line: 0,

            curr_token: Token::None,
        })
    }

    fn read_line(&mut self) {
        let mut line_buffer = String::new();

        if let Err(_) = self.file_reader.read_line(&mut line_buffer) {
            self.token_buffer.push_back( Token::Eof );
            return;
        }

        if line_buffer.is_empty() {
            self.token_buffer.push_back( Token::Eof );
            return;
        }

        let line = line_buffer.trim();
        self.line += 1;

        let mut word_no = 1;

        for word in line.split_ascii_whitespace().into_iter() {
            if word.starts_with('@') {
                self.token_buffer.push_back(
                    Token::Label(
                        Rc::new(String::from( &word[1..] )), 
                        Position::new(self.line, word_no) 
                    ) 
                );
            }
            else if word.starts_with('%') {
                self.token_buffer.push_back(
                    Token::Register(
                        Rc::new(String::from( &word[1..] )),
                        Position::new(self.line, word_no)
                    )
                );
            }
            else {
                self.token_buffer.push_back(
                    Token::Op(
                        Rc::new(String::from(word)),
                        Position::new(self.line, word_no)
                    )
                );
            }

            word_no += 1;
        }

        self.token_buffer.push_back( Token::NewLine(self.line) );
    }

    pub fn next(&mut self) -> Token {
        if let Some(tok) = self.token_buffer.pop_front() {
            self.curr_token = tok;

            return self.curr();
        }

        self.read_line();

        let tok = self.token_buffer.pop_front().unwrap();
        self.curr_token = tok;
        
        return self.curr();
    }

    pub fn curr(&self) -> Token {
        return self.curr_token.clone();
    }
}