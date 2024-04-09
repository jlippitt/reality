pub use ex::{cop2, lwc2, swc2};
pub use regs::Vector;

use super::{Core, DfState};
use regs::{Accumulator, FlagVector, Flags};
use tracing::trace;

mod ex;
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

    pub fn reg(&self, index: usize) -> Vector {
        self.regs[index]
    }

    pub fn set_reg(&mut self, index: usize, value: Vector) {
        self.regs[index] = value;
        trace!("  V{:02}: {}", index, self.regs[index]);
    }
}
