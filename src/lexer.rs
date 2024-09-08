use std::io::{self, BufRead};
use std::rc::Rc;
use std::{fs::File, io::BufReader};
use std::collections::LinkedList;
use regex::Regex;

const COMMENT_CHAR: char = '#';

pub type LexPosType = u32;
pub type LexStr = Rc<String>;

#[derive(Clone, Copy, Debug)]
pub struct LexPos {
    line: LexPosType,
    sym: LexPosType
}

impl LexPos {
    pub fn new(line: LexPosType, sym: LexPosType) -> Self {
        LexPos {
            line: line,
            sym: sym
        }
    }

    pub fn str(&self) -> String {
        format!("{}:{}", self.line, self.sym)
    }

    pub fn line(&self) -> LexPosType {
        self.line
    }

    pub fn sym(&self) -> LexPosType {
        self.sym
    }
}

#[derive(Clone, Debug)]
pub enum Token {
    None,
    Eof,
    NewLine(LexPos), // line

    Dot(LexPos),
    At(LexPos),
    Percent(LexPos),
    BrOpen(LexPos),
    BrClose(LexPos),
    Pipe(LexPos),
    
    Macro(LexPos),

    Id(LexStr, LexPos),
    Unknown(LexPos),
}

impl Token {
    pub fn pos_str(&self) -> String {
        match self {
            Token::None => String::from("?:?"),
            Token::Eof => String::from("-1:-1"),

            Token::NewLine(pos)
            | Token::Dot(pos)
            | Token::At(pos)
            | Token::Percent(pos)
            | Token::BrOpen(pos)
            | Token::BrClose(pos)
            | Token::Pipe(pos)
            | Token::Macro(pos)
            | Token::Id(_, pos)
            | Token::Unknown(pos) => format!("{}:{}", pos.line(), pos.sym()),
        }
    }

    pub fn str(&self) -> String {
        match self {
            Token::None => String::from("None"),
            Token::Eof => String::from("EOF"),
            Token::NewLine(_) => String::from("NewLine"),
            Token::Unknown(_) => String::from("Unknown"),
            Token::Dot(_) => String::from("'.'"),
            Token::At(_) => String::from("'@'"),
            Token::BrOpen(_) => String::from("'{'"),
            Token::BrClose(_) => String::from("'}'"),
            Token::Pipe(_) => String::from("'|'"),
            Token::Percent(_) => String::from("'%'"),
            Token::Macro(_) => String::from("'macro'"),
            Token::Id(name, _) => format!("Id({})", name),
        }
    }
}


pub struct Lexer {
    file_reader: BufReader<File>,
    token_buffer: LinkedList<Token>, // read whole line and place to buffer
    line: LexPosType,

    curr_token: Token,

    id_regex: Regex,
}

impl Lexer {
    pub fn new(path: &str) -> io::Result<Self> {
        Ok(Lexer {
            file_reader: BufReader::new(File::open(path)?),
            token_buffer: LinkedList::new(),
            line: 0,

            curr_token: Token::None,

            id_regex: Regex::new("^[0-9a-zA-Z_]+").unwrap(),
        })
    }

    fn read_line(&mut self) {
        let mut line_buffer = String::new();

        self.file_reader.read_line(&mut line_buffer).unwrap();

        if line_buffer.is_empty() {
            self.token_buffer.push_back(Token::Eof);
            return;
        };       

        let mut line = line_buffer.as_str();
        if let Some(i) = line.find(COMMENT_CHAR) {
            line = &line[..i];
        }

        self.line += 1;
        let mut sym_no = 1;

        while !line.is_empty() {
            if line.starts_with(' ') || line.starts_with('\t') || line.starts_with('\n')  {
                sym_no += 1;
                line = &line[1..];
            }
            else if line.starts_with(';') {
                self.token_buffer.push_back(Token::NewLine(LexPos::new(self.line, sym_no)));
                sym_no += 1;
                line = &line[1..];
            }
            else if line.starts_with('.') {
                self.token_buffer.push_back(Token::Dot(LexPos::new(self.line, sym_no)));
                sym_no += 1;
                line = &line[1..];
            }
            else if line.starts_with('@') {
                self.token_buffer.push_back(Token::At(LexPos::new(self.line, sym_no)));
                sym_no += 1;
                line = &line[1..];
            }
            else if line.starts_with('%') {
                self.token_buffer.push_back(Token::Percent(LexPos::new(self.line, sym_no)));
                sym_no += 1;
                line = &line[1..];
            }
            else if line.starts_with('{') {
                self.token_buffer.push_back(Token::BrOpen(LexPos::new(self.line, sym_no)));
                sym_no += 1;
                line = &line[1..];
            }
            else if line.starts_with('}') {
                self.token_buffer.push_back(Token::BrClose(LexPos::new(self.line, sym_no)));
                sym_no += 1;
                line = &line[1..];
            }
            else if line.starts_with('|') {
                self.token_buffer.push_back(Token::Pipe(LexPos::new(self.line, sym_no)));
                sym_no += 1;
                line = &line[1..];
            }
            else if let Some(caps) = self.id_regex.captures(line) {
                let id = &caps[0];
                let pos = LexPos::new(self.line, sym_no);

                match id {
                    "macro" => self.token_buffer.push_back(Token::Macro(pos)),
                    other => self.token_buffer.push_back(Token::Id( Rc::new(String::from(other)), pos ))
                }

                let d = id.len();
                sym_no += d as u32;
                line = &line[d..];
            }
            else {
                self.token_buffer.push_back(Token::Unknown(LexPos::new(self.line, sym_no)));
                return;
            }
        }

        self.token_buffer.push_back(Token::NewLine(LexPos::new(self.line, sym_no)));
    }

    pub fn next(&mut self) -> Token {
        loop {
            let r = self.token_buffer.pop_front();

            match r {
                Some(tok) => {
                    self.curr_token = tok;
                    break;
                },
                None => {
                    self.read_line();
                },
            }
        }
        
        self.curr()
    }

    pub fn curr(&self) -> Token {
        return self.curr_token.clone();
    }
}