
type NatBase = u64;

#[derive(Clone, PartialEq, Eq)]
struct Node {
    pub value: NatBase,
    next: Option<Box<Node>>
}

impl Node {
    pub fn zero() -> Self {
        Node {
            value: 0,
            next: None
        }
    }

    pub fn one() -> Self {
        Node {
            value: 1,
            next: None
        }
    }

    pub fn increment(&mut self) {
        self.value += 1;

        if self.value == 0 {
            match &mut self.next {
                Some(n) => n.increment(),
                None => self.next = Some(Box::new(Node::one())),
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct BigNat {
    begin: Node
}

impl BigNat {
    pub fn new() -> Self {
        BigNat {
            begin: Node::zero()
        }
    }

    pub fn increment(&mut self) {
        self.begin.increment();
    }

    pub fn zero(&mut self) {
        self.begin = Node::zero();
    }

    pub fn print(&self) {
        let c = char::from_u32(self.begin.value as u32).unwrap();
        print!("{}", c); // TODO: if greater not print
    }
}