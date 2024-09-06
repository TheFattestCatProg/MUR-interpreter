use crate::bignat::BigNat;

pub type CellType = usize;
pub type PosType = usize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Zero(CellType),
    Inc(CellType),
    Mov(CellType, CellType),
    Jmp(CellType, CellType, PosType), // r1, r2, label
    Out(CellType),
}

struct Env {
    regs: Vec<BigNat>,
    zero: BigNat
}

impl Env {
    pub fn new() -> Self {
        Env {
            regs: Vec::new(),
            zero: BigNat::new()
        }
    }

    pub fn reg(&self, r: CellType) -> &BigNat {
        if r < self.regs.len() {
            &self.regs[r]
        }
        else {
            &self.zero
        }
    }

    pub fn reg_mut(&mut self, r: CellType) -> &mut BigNat {
        while r >= self.regs.len() {
            self.regs.push(BigNat::new());
        }

        return &mut self.regs[r];
    }
}

pub fn execute(code: Vec<Op>) {
    let mut env = Env::new();
    let mut ip: PosType = 0;


    while ip < code.len() {
        match code[ip] {
            Op::Zero(r) => env.reg_mut(r).zero(),
            Op::Inc(r) => env.reg_mut(r).increment(),
            Op::Mov(r1, r2) => *env.reg_mut(r1) = env.reg(r2).clone(),
            Op::Jmp(r1, r2, new_ip) => if env.reg(r1) == env.reg(r2) { ip = new_ip - 1; },
            Op::Out(r) => env.reg(r).print(),
        }

        ip += 1;
    }
}