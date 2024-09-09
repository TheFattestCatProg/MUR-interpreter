use std::collections::{HashMap, LinkedList};

use crate::{lexer::Lexer, meta::parse_to_meta, meta2::{print_meta2, to_meta2, CodeMeta2, Meta2}, vm::{Op, PosType}};


pub struct Parser {
    lexer: Lexer,
}


fn meta2_to_vm(meta2: LinkedList<Meta2>) -> Result<Vec<Op>, String> {
    let mut labels_pos = HashMap::new();
    let mut op_pointer: PosType = 0;

    for i in meta2.iter() {
        match i {
            Meta2::Lab(name, pos) => {
                if let Some(_) = labels_pos.insert(name, op_pointer) {
                    return Err(format!("{} Label '{}{}' is already defined", pos.str(), if name.param() != 0 { "." } else { "" }, name.id()))
                }
            },
            _ => op_pointer += 1,
        }
    }

    let mut vec = Vec::with_capacity(op_pointer);

    for i in meta2.iter() {
        match i {
            Meta2::Lab(_, _) => (),
            Meta2::Zer(r) => vec.push(Op::Zero(*r)),
            Meta2::Inc(r) => vec.push(Op::Inc(*r)),
            Meta2::Out(r) => vec.push(Op::Out(*r)),
            Meta2::Mov(r1, r2) => vec.push(Op::Mov(*r1, *r2)),
            Meta2::Jmp(r1, r2, l, pos) => {
                let Some(v) = labels_pos.get(&l) else {
                    return Err(format!("{} Label '{}{}' not found", pos.str(), if l.param() != 0 { "." } else { "" }, l.id()))
                };

                vec.push(Op::Jmp(*r1, *r2, *v));
            },
        }
    }

    return Ok(vec);
}


impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser {
            lexer: lexer,
        }
    }

    pub fn parse_meta2(&mut self) -> Result<CodeMeta2, String> {
        let meta_res = parse_to_meta(&mut self.lexer)?;
        Ok(to_meta2(meta_res.code, meta_res.macros)?)
    }

    pub fn print_debug(&mut self) -> Result<(), String> {
        print_meta2(&self.parse_meta2()?);
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Vec<Op>, String> {
        let meta2 = self.parse_meta2()?;
        meta2_to_vm(meta2)
    }
}