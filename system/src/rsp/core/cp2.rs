use regs::{Accumulator, FlagVector, Vector};

mod regs;

pub struct Cp2 {
    regs: [Vector; 32],
    acc: Accumulator,
    flags: FlagVector,
}

impl Cp2 {
    pub fn new() -> Self {
        Self {
            regs: Default::default(),
            acc: Accumulator::default(),
            flags: FlagVector::default(),
        }
    }
}
