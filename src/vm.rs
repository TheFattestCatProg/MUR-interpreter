use crate::bignat::BigNat;

pub type CellType = usize;
pub type PosType = usize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    ZERO(CellType),
    INC(CellType),
    MOV(CellType, CellType),
    JMP(CellType, CellType, PosType) // r1, r2, label
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
            Op::ZERO(r) => env.reg_mut(r).zero(),
            Op::INC(r) => env.reg_mut(r).increment(),
            Op::MOV(r1, r2) => *env.reg_mut(r1) = env.reg(r2).clone(),
            Op::JMP(r1, r2, new_ip) => if env.reg(r1) == env.reg(r2) { ip = new_ip - 1; },
        }

        ip += 1;
    }
}