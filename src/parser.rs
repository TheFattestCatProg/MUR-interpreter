use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

use crate::lexer::{Lexer, Position, Token};
use crate::vm::{CellType, Op, PosType};

enum Meta {
    Op(Rc<String>, LinkedList<MetaArg>, Position),
    Label(Rc<String>),
}

enum MetaArg {
    Register(Rc<String>, Position),
    Label(Rc<String>, Position)
}

struct Env {
    labels: HashMap<Rc<String>, PosType>,
    regs: HashMap<Rc<String>, CellType>,

    next_reg: CellType,

    code: LinkedList<Op>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            labels: HashMap::new(),
            regs: HashMap::new(),

            next_reg: 0,

            code: LinkedList::new()
        }
    }
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser {
            lexer: lexer,
        }
    }

    fn parse_to_meta(&mut self) -> Result<LinkedList<Meta>, String> {
        let mut meta_list = LinkedList::new();

        fn bad_token(tok: &Token) -> String {
            return format!("{} Bad token: {}", tok.pos_str(), tok.str())
        }

        fn parse_op_args(lexer: &mut Lexer) -> Result<LinkedList<MetaArg>, String> {
            let mut args = LinkedList::new();

            loop {
                let tok = lexer.next();

                match tok {
                    Token::NewLine(_) => return Ok(args),

                    Token::None | Token::Eof | Token::Op(_, _) => return Err(bad_token(&tok)),

                    Token::Register(name, pos) => args.push_back(MetaArg::Register(name, pos)),
                    Token::Label(name, pos)    => args.push_back(MetaArg::Label(name, pos))
                }
            }
        }

        loop {
            let tok = self.lexer.next();

            match tok {
                Token::None           => return Err(bad_token(&tok)),
                Token::Register(_, _) => return Err(bad_token(&tok)),
                Token::NewLine(_)     => continue,
                Token::Eof            => return Ok(meta_list),

                Token::Op(name, pos) => meta_list.push_back(Meta::Op(name, parse_op_args(&mut self.lexer)?, pos)),
                Token::Label(name, _) => { 
                    meta_list.push_back(Meta::Label(name));

                    match self.lexer.next() {
                        Token::NewLine(_) => (),
                        _ => return Err(bad_token(&self.lexer.curr())),
                    }
                },
            }
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Op>, String> {
        let meta = self.parse_to_meta()?;

        let mut env = Env::new();
        let mut op_counter: PosType = 0;

        for i in meta.iter() {
            match i {
                Meta::Op(_, _, _) => op_counter += 1,
                Meta::Label(name) => { env.labels.insert(Rc::clone(name), op_counter); },
            }
        }

        fn label_to_num(env: &Env, name: &Rc<String>, pos: &Position) -> Result<PosType, String> {
            match env.labels.get(name) {
                Some(pos) => Ok(*pos),
                None => Err(format!("{} Label @{} not found", pos.str(), name)),
            }
        }

        fn reg_to_num(env: &mut Env, name: &Rc<String>) -> CellType {
            if let Some(t) = env.regs.get(name) {
                return *t;
            }
            
            let old_num = env.next_reg;
            env.regs.insert(Rc::clone(name), old_num);
            env.next_reg += 1;

            old_num
        }

        fn to_op(env: &mut Env, op_name: &Rc<String>, args: &LinkedList<MetaArg>, pos: &Position) -> Result<Op, String> {
            let mut iter = args.iter();

            fn expected_register(name: &Rc<String>, pos: &Position) -> Result<Op, String> {
                Err(format!("{} Expected register, got @{}", pos.str(), name))
            }

            fn expected_label(name: &Rc<String>, pos: &Position) -> Result<Op, String> {
                Err(format!("{} Expected register, got %{}", pos.str(), name))
            }

            fn expected_argument(pos: &Position) -> Result<Op, String> {
                Err(format!("{} Expected argument", pos.str()))
            }

            let op = match op_name.as_str() {
                "zer" => {
                    let reg = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, _) => reg_to_num(env, name),
                            MetaArg::Label(name, pos) => return expected_register(name, pos),
                        },
                    };

                    Op::Zero(reg)
                },
                "inc" => {
                    let reg = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, _) => reg_to_num(env, name),
                            MetaArg::Label(name, pos) => return expected_register(name, pos),
                        },
                    };

                    Op::Inc(reg)
                },
                "out" => {
                    let reg = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, _) => reg_to_num(env, name),
                            MetaArg::Label(name, pos) => return expected_register(name, pos),
                        },
                    };

                    Op::Out(reg)
                },
                "mov" => {
                    let reg1 = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, _) => reg_to_num(env, name),
                            MetaArg::Label(name, pos) => return expected_register(name, pos),
                        },
                    };
                    let reg2 = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, _) => reg_to_num(env, name),
                            MetaArg::Label(name, pos) => return expected_register(name, pos),
                        },
                    };

                    Op::Mov(reg1, reg2)
                },
                "jmp" => {
                    let reg1 = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, _) => reg_to_num(env, name),
                            MetaArg::Label(name, pos) => return expected_register(name, pos),
                        },
                    };
                    let reg2 = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, _) => reg_to_num(env, name),
                            MetaArg::Label(name, pos) => return expected_register(name, pos),
                        },
                    };
                    let label = match iter.next() {
                        None => return expected_argument(pos),
                        Some(m) => match m {
                            MetaArg::Register(name, pos) => return expected_label(name, pos),
                            MetaArg::Label(name, pos) => label_to_num(env, name, pos)?,
                        },
                    };


                    Op::Jmp(reg1, reg2, label)
                },
                other => return Err(format!("{} Bad command '{}'", pos.str(), other))
            };

            if let Some(m) = iter.next() {
                return Err(match m {
                    MetaArg::Register(name, pos) => format!("{} Extra register %{}", pos.str(), name),
                    MetaArg::Label(name, pos) => format!("{} Extra label @{}", pos.str(), name),
                });
            }

            Ok(op)
        }

        for m in meta.iter() {
            match m {
                Meta::Label(_) => (),
                Meta::Op(name, args, pos) => {
                    let op = to_op(&mut env, name, args, pos)?;
                    env.code.push_back(op);
                },
            }
        }

        let mut vec = Vec::with_capacity(env.code.len());

        while !env.code.is_empty() {
            vec.push(env.code.pop_front().unwrap());
        }

        return Ok(vec);
    }
}