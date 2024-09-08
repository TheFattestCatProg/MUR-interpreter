use std::collections::linked_list::IntoIter;
use std::collections::{HashMap, LinkedList};
use std::hash::Hash;
use std::rc::Rc;

use crate::lexer::{LexPos, LexStr};
use crate::meta::{CodeMeta, IsLocal, MacroData, Meta, MetaArg, MetaArgs, NonLocalSearch};
use crate::vm::CellType;

struct ViewSpace<T, V> {
    levels: LinkedList<HashMap<T, V>>
}

impl<T, V> ViewSpace<T, V>
where T: Hash, T: Eq 
{
    pub fn new() -> Self {
        ViewSpace {
            levels: LinkedList::new()
        }
    }

    pub fn push_level(&mut self) {
        self.levels.push_front(HashMap::new())
    }

    pub fn push_from(&mut self, from: HashMap<T, V>) {
        self.levels.push_front(from)
    }

    pub fn pop_level(&mut self) -> HashMap<T, V> {
        self.levels.pop_front().unwrap()
    }

    pub fn find(&self, name: &T) -> Option<&V> {
        for i in self.levels.iter() {
            if let Some(r) = i.get(name) {
                return Some(r);
            }
        }

        None
    }

    pub fn find_top(&self, name: &T) -> Option<&V> {
        if let Some(r) = self.levels.front().unwrap().get(name) {
            return Some(r)
        }

        None
    }

    pub fn find_global(&self, name: &T) -> Option<&V> {
        if let Some(r) = self.levels.back().unwrap().get(name) {
            return Some(r)
        }

        None
    }

    pub fn put(&mut self, name: T, value: V) -> Result<(), ()> {
        let front = self.levels.front_mut().unwrap();
        if let Some(_) = front.insert(name, value) {
            return Err(())
        }

        Ok(())
    }

    pub fn put_global(&mut self, name: T, value: V) -> Result<(), ()> {
        if let Some(_) = self.levels.back_mut().unwrap().insert(name, value) {
            return Err(())
        }

        Ok(())
    }
}

type ParamType = u64;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MetaId {
    id: LexStr,
    param: ParamType
}

impl MetaId {
    pub fn new(id: LexStr, param: ParamType) -> Self {
        MetaId {
            id: id,
            param: param
        }
    }

    pub fn id(&self) -> &LexStr {
        return &self.id;
    }

    pub fn param(&self) -> ParamType {
        return self.param;
    }
}

#[derive(Debug, Clone)]
pub enum Meta2 {
    Lab(MetaId, LexPos),
    Zer(CellType),
    Inc(CellType),
    Out(CellType),
    Mov(CellType, CellType),
    Jmp(CellType, CellType, MetaId, LexPos)
}

pub type CodeMeta2 = LinkedList<Meta2>;

#[derive(Debug, Clone)]
enum Meta2Arg {
    Reg(CellType),
    Lab(MetaId)
}

struct Env {
    next_reg: CellType,
    free_regs: LinkedList<CellType>,
    next_param: ParamType,

    macro_space: ViewSpace<LexStr, MacroData>,
    reg_space: ViewSpace<LexStr, CellType>,
    replacements_space: ViewSpace<LexStr, Meta2Arg>
}

impl Env {
    pub fn new(global_macros: HashMap<LexStr, MacroData>) -> Self {
        let mut macro_space = ViewSpace::new();
        let mut reg_space = ViewSpace::new();
        let mut repl_space = ViewSpace::new();

        macro_space.push_from(global_macros);
        reg_space.push_level();
        repl_space.push_level();

        Env {
            next_reg: 0,
            free_regs: LinkedList::new(),
            next_param: 0,

            macro_space: macro_space,
            reg_space: reg_space,
            replacements_space: repl_space,
        }
    }

    pub fn next_param(&mut self) -> ParamType {
        self.next_param += 1;
        return self.next_param;
    }

    pub fn get_reg(&mut self, name: LexStr, is_local: IsLocal, non_local_search: NonLocalSearch) -> CellType {
        if is_local {
            if non_local_search {
                if let Some(r) = self.reg_space.find(&name) {
                    return *r;
                }
            }
            else {
                if let Some(r) = self.reg_space.find_top(&name) {
                    return *r;
                }
            }
        }
        else {
            if let Some(r) = self.reg_space.find_global(&name) {
                return *r;
            }
        }

        if let Some(r) = self.free_regs.pop_front() {
            if is_local {
                self.reg_space.put(name, r).unwrap();
                
            }
            else {
                self.reg_space.put_global(name, r).unwrap();
            }
            return r;
        }

        let new_reg = self.next_reg;
        self.next_reg += 1;

        if is_local {
            self.reg_space.put(name, new_reg).unwrap();
        }
        else {
            self.reg_space.put_global(name, new_reg).unwrap();
        }
        
        return new_reg;
    }

    pub fn get_macro(&mut self, name: &LexStr) -> Option<&MacroData> {
        let Some(res) = self.macro_space.find(name) else {
            return None
        };

        Some(res)
    }

    pub fn push_level(&mut self, macros: HashMap<LexStr, MacroData>, replacements: HashMap<LexStr, Meta2Arg>) {
        self.reg_space.push_level();
        self.macro_space.push_from(macros);
        self.replacements_space.push_from(replacements);
    }

    pub fn pop_level(&mut self) {
        for i in self.reg_space.pop_level().iter() {
            self.free_regs.push_back(*i.1);
        }
        self.macro_space.pop_level();
        self.replacements_space.pop_level();
    }

    pub fn replace(&self, id: &LexStr) -> Option<&Meta2Arg> {
        self.replacements_space.find(id)
    }
}

const PARAM_GLOBAL: ParamType = 0;


fn expected_arg(pos: LexPos) -> String {
    format!("{} Expected argument", pos.str())
}

fn expected_register(pos: LexPos) -> String {
    format!("{} Expected register", pos.str())
}

fn expected_label(pos: LexPos) -> String {
    format!("{} Expected label", pos.str())
}

fn process_label(name: LexStr, deep_level: ParamType, pos: LexPos) -> Meta2 {
    Meta2::Lab(MetaId::new(name, deep_level), pos)
}

fn arg_to_meta(env: &mut Env, param: ParamType, meta: MetaArg) -> Result<(Meta2Arg, LexPos), String> {
    match meta {
        MetaArg::Reg(name, is_local, nls,  pos) => Ok(( Meta2Arg::Reg(env.get_reg(name, is_local, nls)), pos )),
        MetaArg::Lab(name, is_local, pos) => Ok(( Meta2Arg::Lab(MetaId::new(name, if is_local { param } else { PARAM_GLOBAL } )), pos )),
        MetaArg::Id(name, pos) => match env.replace(&name) {
            None => Err(format!("{} Cannot expand '{}'", pos.str(), name)),
            Some(val) => Ok((val.clone(), pos)),
        },
        MetaArg::Code(_, _, pos) => Err(format!("{} Expected non-macro argument", pos.str())),
    }
}

fn process_zer(env: &mut Env, param: ParamType, arg_iter: &mut IntoIter<MetaArg>, pos: LexPos) -> Result<Meta2, String> {
    let arg1 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(v), _) => v,
            (Meta2Arg::Lab(_), pos) => return Err(expected_register(pos)),
        }
    };

    Ok(Meta2::Zer(arg1))
}

fn process_inc(env: &mut Env, param: ParamType, arg_iter: &mut IntoIter<MetaArg>, pos: LexPos) -> Result<Meta2, String> {
    let arg1 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(v), _) => v,
            (Meta2Arg::Lab(_), pos) => return Err(expected_register(pos)),
        }
    };

    Ok(Meta2::Inc(arg1))
}

fn process_out(env: &mut Env, param: ParamType, arg_iter: &mut IntoIter<MetaArg>, pos: LexPos) -> Result<Meta2, String> {
    let arg1 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(v), _) => v,
            (Meta2Arg::Lab(_), pos) => return Err(expected_register(pos)),
        }
    };

    Ok(Meta2::Out(arg1))
}

fn process_mov(env: &mut Env, param: ParamType, arg_iter: &mut IntoIter<MetaArg>, pos: LexPos) -> Result<Meta2, String> {
    let arg1 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(v), _) => v,
            (Meta2Arg::Lab(_), pos) => return Err(expected_register(pos)),
        }
    };
    let arg2 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(v), _) => v,
            (Meta2Arg::Lab(_), pos) => return Err(expected_register(pos)),
        }
    };

    Ok(Meta2::Mov(arg1, arg2))
}

fn process_jmp(env: &mut Env, param: ParamType, arg_iter: &mut IntoIter<MetaArg>, pos: LexPos) -> Result<Meta2, String> {
    let arg1 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(v), _) => v,
            (Meta2Arg::Lab(_), pos) => return Err(expected_register(pos)),
        }
    };
    let arg2 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(v), _) => v,
            (Meta2Arg::Lab(_), pos) => return Err(expected_register(pos)),
        }
    };
    let arg3 = match arg_iter.next() {
        None => return Err(expected_arg(pos)),
        Some(meta) => match arg_to_meta(env, param, meta)? {
            (Meta2Arg::Reg(_), pos) => return Err(expected_label(pos)),
            (Meta2Arg::Lab(v), _) => v,
        }
    };

    Ok(Meta2::Jmp(arg1, arg2, arg3, pos))
}

fn expand_macro(env: &mut Env, param: ParamType, name: LexStr, args: &mut IntoIter<MetaArg>, pos: LexPos) -> Result<CodeMeta2, String> {
    let inner_param = env.next_param();

    let Some((ids, code)) = env.get_macro(&name).cloned() else {
        return Err(format!("{} Cannot find macro '{}'", pos.str(), name));
    };

    let mut macr = HashMap::new();
    let mut reps = HashMap::new();

    for i in ids.iter() {
        match args.next() {
            None => return Err(expected_arg(pos)),
            Some(arg) => match arg {
                MetaArg::Reg(name, is_local, nls, _) => {
                    reps.insert(Rc::clone(i), Meta2Arg::Reg( env.get_reg(name, is_local, nls) ));
                },
                MetaArg::Lab(name, is_local, _) => {
                    reps.insert(Rc::clone(i), Meta2Arg::Lab( MetaId::new(name, if is_local { param } else { PARAM_GLOBAL } ) ));
                },
                MetaArg::Code(ids, code, _) => {
                    macr.insert(Rc::clone(i), (ids, code));
                }
                MetaArg::Id(name, pos) => match env.replace(&name) {
                    Some(v) => { reps.insert(Rc::clone(i), v.clone()); },
                    None => match env.get_macro(&name) {
                        Some(data) => { 
                            macr.insert(Rc::clone(i), data.clone()); 
                        },
                        None => return Err(format!("{} Cannot expand '{}'", pos.str(), name)),
                    },
                },
            },
        }
    }

    env.push_level(macr, reps);

    let mut to_ret = CodeMeta2::new();

    for i in code.into_iter() {
        match i {
            Meta::Op(name, op_args, op_pos) => to_ret.append(&mut process_op(env, inner_param, name, op_args, op_pos)?),
            Meta::Lab(name, is_local, pos) => to_ret.push_back(Meta2::Lab(MetaId::new(name, if is_local { inner_param } else { PARAM_GLOBAL } ), pos))
        }
    }

    env.pop_level();

    Ok(to_ret)
}

fn process_op(env: &mut Env, param: ParamType, name: LexStr, args: MetaArgs, pos: LexPos) -> Result<CodeMeta2, String> {
    let mut to_ret = CodeMeta2::new();
    let mut iter = args.into_iter();
    
    match name.as_str() {
        "zer" => to_ret.push_back(process_zer(env, param, &mut iter, pos)?),
        "inc" => to_ret.push_back(process_inc(env, param, &mut iter, pos)?),
        "out" => to_ret.push_back(process_out(env, param, &mut iter, pos)?),

        "mov" => to_ret.push_back(process_mov(env, param, &mut iter, pos)?),
        "jmp" => to_ret.push_back(process_jmp(env, param, &mut iter, pos)?),

        _ => to_ret.append(&mut expand_macro(env, param, name, &mut iter, pos)?),
    }

    if let Some(arg) = iter.next() {
        return Err(format!("{} Extra argument: {}", arg.pos_str(), arg.str()))
    }

    Ok(to_ret)
}

pub fn to_meta2(meta: CodeMeta, macros: HashMap<LexStr, MacroData>) -> Result<CodeMeta2, String> {
    let mut env = Env::new(macros);
    let mut meta2 = CodeMeta2::new();

    for i in meta.into_iter() {
        match i {
            Meta::Op(name, args, pos) => meta2.append(
                &mut process_op(&mut env, 0, name, args, pos)?
            ),
            Meta::Lab(name, _, pos) => meta2.push_back(
                process_label(name, PARAM_GLOBAL, pos)
            ),
        }
    }

    Ok(meta2)
}

pub fn print_meta2(meta: &CodeMeta2) {
    for i in meta.iter() {
        match i {
            Meta2::Lab(id, _) => println!("@{}_{}", id.id(), id.param()),
            Meta2::Zer(r) => println!("zer %{}", *r),
            Meta2::Inc(r) => println!("inc %{}", *r),
            Meta2::Out(r) => println!("out %{}", *r),
            Meta2::Mov(r1, r2) => println!("mov %{} %{}", *r1, *r2),
            Meta2::Jmp(r1, r2, l, _) => println!("jmp %{} %{} @{}_{}", *r1, *r2, l.id(), l.param()),
        }
    }
}