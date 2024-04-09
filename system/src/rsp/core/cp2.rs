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
    const CONTROL_REG_NAMES: [&'static str; 32] = [
        "VCO", "VCC", "VCE", "VC3", "VC4", "VC5", "VC6", "VC7", "VC8", "VC9", "VC10", "VC11",
        "VC12", "VC13", "VC14", "VC15", "VC16", "VC17", "VC18", "VC19", "VC20", "VC21", "VC22",
        "VC23", "VC24", "VC25", "VC26", "VC27", "VC28", "VC29", "VC30", "VC31",
    ];

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

    pub fn control_reg(&self, index: usize) -> i32 {
        let value = match index {
            0 => u16::from_le_bytes([
                self.flags.read(Flags::CARRY),
                self.flags.read(Flags::NOT_EQUAL),
            ]),
            1 => u16::from_le_bytes([
                self.flags.read(Flags::COMPARE),
                self.flags.read(Flags::CLIP_COMPARE),
            ]),
            2 => u16::from_le_bytes([self.flags.read(Flags::COMPARE_EXTENSION), 0]),
            reg => todo!(
                "RSP CP2 Control Register Read: {}",
                Self::CONTROL_REG_NAMES[reg]
            ),
        };

        // TODO: Verify that this is actual behaviour
        value as i16 as i32
    }
}
