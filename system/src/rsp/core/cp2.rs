pub use ex::{cop2, lwc2, swc2};
pub use regs::Vector;

use super::{Core, DfState};
use regs::{Accumulator, FlagVector, Flags};
use std::array;
use tracing::trace;

mod ex;
mod regs;

const LOOKUP_TABLE_SIZE: usize = 512;

pub struct Cp2 {
    regs: [Vector; 32],
    acc: Accumulator,
    flags: FlagVector,
    div_in: u32,
    div_out: u32,
    reciprocal: [u16; LOOKUP_TABLE_SIZE],
    inv_sqrt: [u16; LOOKUP_TABLE_SIZE],
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
            div_in: 0,
            div_out: 0,
            reciprocal: array::from_fn(reciprocal),
            inv_sqrt: array::from_fn(inv_sqrt),
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

fn reciprocal(index: usize) -> u16 {
    if index == 0 {
        return 0xffff;
    }

    ((((1u64 << 34) / (index as u64 + 512)) + 1) >> 8) as u16
}

fn inv_sqrt(index: usize) -> u16 {
    let input = (index as u64 + 512) >> (index as u64 & 1);
    let mut result = 1u64 << 17;

    while (input * (result + 1) * (result + 1)) < (1u64 << 44) {
        result += 1;
    }

    (result >> 1) as u16
}
