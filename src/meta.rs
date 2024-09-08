use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

use crate::lexer::{LexPos, LexStr, Lexer, Token};

pub type IsLocal = bool;
pub type NonLocalSearch = bool;

#[derive(Debug, Clone)]
pub enum MetaArg {
    Reg(LexStr, IsLocal, NonLocalSearch, LexPos),
    Lab(LexStr, IsLocal, LexPos),
    Id(LexStr, LexPos),
    Code(LinkedList<LexStr>, CodeMeta, LexPos),
}

impl MetaArg {
    pub fn pos_str(&self) -> String {
        match self {
            MetaArg::Reg(_, _, _, pos)
            | MetaArg::Lab(_, _, pos)
            | MetaArg::Id(_, pos)
            | MetaArg::Code(_, _, pos) => pos.str(),
        }
    }

    pub fn str(&self) -> String {
        match self {
            MetaArg::Reg(name, is_local, search, _) => format!("%{}{}{}", if *is_local { "." } else { "" }, if *search { "\\" } else { "" }, name),
            MetaArg::Lab(name, is_local, _) => format!("@{}{}", if *is_local { "." } else { "" }, name),
            MetaArg::Id(name, _) => format!("Id:{}", name),
            MetaArg::Code(_, _, _) => String::from("{...code...}"),
        }
    }
}

pub type MetaArgs = LinkedList<MetaArg>;

#[derive(Debug, Clone)]
pub enum Meta {
    Op(LexStr, MetaArgs, LexPos),
    Lab(LexStr, IsLocal, LexPos)
}

pub type CodeMeta = LinkedList<Meta>;
pub type MacroData = (LinkedList<LexStr>, CodeMeta);

pub struct ParsMetaResult {
    pub code: CodeMeta,
    pub macros: HashMap<LexStr, MacroData>
}

fn bad_token(tok: Token) -> String {
    format!("{} Bad token: {}", tok.pos_str(), tok.str())
}

fn parse_arg_reg(lexer: &mut Lexer) -> Result<MetaArg, String> {
    match lexer.next() {       
        Token::Id(name, pos) => return Ok(MetaArg::Reg(name, false, false, pos)),
        Token::Dot(_) => match lexer.next() {
            Token::Id(name, pos) => Ok(MetaArg::Reg(name, true, false, pos)),
            tok => Err(bad_token(tok)),
        },
        Token::InvSlash(_) => match lexer.next() {
            Token::Dot(_) => match lexer.next() {
                Token::Id(name, pos) => Ok(MetaArg::Reg(name, true, true, pos)),
            tok => Err(bad_token(tok)),
            },
            tok => Err(bad_token(tok))
        }
        tok => return Err(bad_token(tok)),
    }
}

fn parse_arg_lab(lexer: &mut Lexer, pos: LexPos) -> Result<MetaArg, String> {
    match lexer.next() {       
        Token::Id(name, _) => return Ok(MetaArg::Lab(name, false, pos)),
        Token::Dot(_) => (),
        tok => return Err(bad_token(tok)),
    }

    match lexer.next() {        
        Token::Id(name, _) => Ok(MetaArg::Lab(name, true, pos)),
        tok => Err(bad_token(tok)),
    }
}

fn parse_arg_code(lexer: &mut Lexer, pos: LexPos) -> Result<MetaArg, String> {
    let mut ids = LinkedList::new();
    let mut meta = CodeMeta::new();

    loop {
        match lexer.next() {            
            Token::Pipe(_) => break,
            Token::Id(name, _) => {
                if ids.contains(&name) {
                    return Err(format!("{} Parameter with name '{}' is already exist", pos.str(), name));
                }
                ids.push_back(name);
            },

            tok => return Err(bad_token(tok)),
        }
    }

    loop {
        match lexer.next() {
            Token::NewLine(_) => continue,
            Token::BrClose(_) => break,
            
            Token::Id(name, pos) => meta.push_back(parse_op(lexer, name, pos)?),
            Token::At(pos) => meta.push_back(parse_label(lexer, pos)?),

            tok => return Err(bad_token(tok)),
        }
    }

    Ok(MetaArg::Code(ids, meta, pos))
}

fn parse_macro(lexer: &mut Lexer) -> Result<(LexStr, MacroData), String> {
    let name = match lexer.next() {
        Token::Id(name, _) => name,
        tok => return Err(bad_token(tok)),
    };

    let mut args = LinkedList::new();
    let mut meta = CodeMeta::new();

    loop {
        match lexer.next() {
            Token::Id(name, pos) => { 
                if args.contains(&name) {
                    return Err(format!("{} Parameter with name '{}' is already exist", pos.str(), name));
                }
                args.push_back(name);
            },
            Token::BrOpen(_) => break,
            tok => return Err(bad_token(tok)),
        }
    }

    loop {
        match lexer.next() {
            Token::Id(name, pos) => meta.push_back(parse_op(lexer, name, pos)?),
            Token::At(pos) => meta.push_back(parse_label(lexer, pos)?),
            Token::BrClose(_) => break,
            Token::NewLine(_) => continue,
            tok => return Err(bad_token(tok)),
        }
    }

    Ok((name, (args, meta)))
}

fn parse_op(lexer: &mut Lexer, name: LexStr, pos: LexPos) -> Result<Meta, String> {
    let mut args = MetaArgs::new();

    loop {
        match lexer.next() {
            Token::Eof
            | Token::NewLine(_) => break,

            Token::Percent(_) => args.push_back(parse_arg_reg(lexer)?),
            Token::At(pos) => args.push_back(parse_arg_lab(lexer, pos)?),
            Token::Id(name, pos) => args.push_back(MetaArg::Id(name, pos)),
            Token::BrOpen(pos) => args.push_back(parse_arg_code(lexer, pos)?),

            tok => return Err(bad_token(tok)),
        }
    }

    Ok(Meta::Op(name, args, pos))
}

fn parse_label(lexer: &mut Lexer, pos: LexPos) -> Result<Meta, String> {
    match lexer.next() {       
        Token::Id(name, _) => {
            match lexer.next() {
                Token::NewLine(_)
                | Token::Eof => return Ok(Meta::Lab(name, false, pos)),
                tok => return Err(bad_token(tok)),
            }
        },
        Token::Dot(_) => (),
        tok => return Err(bad_token(tok)),
    }

    match lexer.next() {        
        Token::Id(name, _) => {
            match lexer.next() {
                Token::NewLine(_)
                | Token::Eof => return Ok(Meta::Lab(name, true, pos)),
                tok => return Err(bad_token(tok)),
            }
        },
        tok => return Err(bad_token(tok)),
    }
}

pub fn parse_to_meta(lexer: &mut Lexer) -> Result<ParsMetaResult, String> {
    let mut meta = LinkedList::new();
    let mut macros = HashMap::new();

    loop {
        match lexer.next() {
            Token::NewLine(_) => continue,
            Token::Eof => break,
            
            Token::Id(name, pos) => meta.push_back(parse_op(lexer, name, pos)?),
            Token::At(pos) => meta.push_back(parse_label(lexer, pos)?),
            Token::Macro(pos) => {
                let (name, data) = parse_macro(lexer)?;

                if let Some(_) = macros.insert(Rc::clone(&name), data) {
                    return Err(format!("{} Macro with name '{}' is already exist", pos.str(), name));
                }
            },

            tok => return Err(bad_token(tok)),
        }
    }

    Ok(ParsMetaResult {
        code: meta,
        macros: macros
    })
}