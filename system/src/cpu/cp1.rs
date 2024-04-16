pub use ex::{cop1, ldc1, lwc1, sdc1, swc1};

use super::{Cpu, DcOperation};
use bytemuck::Pod;
use regs::Status;
use tracing::trace;

mod ex;
mod regs;

pub struct RegWrite {
    pub reg: usize,
    pub value: i64,
}

pub trait Format: Pod {
    const NAME: &'static str;
    fn cp1_reg(cpu: &Cpu, reg: usize) -> Self;
    fn set_cp1_reg(cpu: &mut Cpu, reg: usize, value: Self) -> RegWrite;
    fn to_f32(self) -> f32;
    fn to_f64(self) -> f64;
}

pub trait Float: Format + num_traits::Float {
    fn round_ties_even(self) -> Self;
    fn to_i32(self) -> i32;
    fn to_i64(self) -> i64;
}

pub trait Int: Format + num_traits::PrimInt {}

pub struct Cp1 {
    status: Status,
}

impl Cp1 {
    pub const REG_OFFSET: usize = 32;

    pub const CONTROL_REG_NAMES: [&'static str; 32] = [
        "Revision", "FCR1", "FCR2", "FCR3", "FCR4", "FCR5", "FCR6", "FCR7", "FCR8", "FCR9",
        "FCR10", "FCR11", "FCR12", "FCR13", "FCR14", "FCR15", "FCR16", "FCR17", "FCR18", "FCR19",
        "FCR20", "FCR21", "FCR22", "FCR23", "FCR24", "FCR25", "FCR26", "FCR27", "FCR28", "FCR29",
        "FCR30", "Status",
    ];

    pub fn new() -> Self {
        Self {
            status: Status::default(),
        }
    }

    pub fn read_control_reg(&self, reg: usize) -> u32 {
        match reg {
            0 => 0x0a00,
            31 => self.status.into(),
            _ => unimplemented!("CP1 Control Reg Read: {:?}", Self::CONTROL_REG_NAMES[reg]),
        }
    }

    pub fn write_control_reg(&mut self, reg: usize, value: u32) {
        match reg {
            0 => (), // FCR0 is read-only
            31 => {
                self.status = (value & 0x0183_ffff).into();
                trace!("  CP1 Status: {:?}", self.status);
            }
            _ => unimplemented!(
                "CP1 Control Reg Write: {:?} <= {:08X}",
                Self::CONTROL_REG_NAMES[reg],
                value
            ),
        }
    }
}

impl Format for i32 {
    const NAME: &'static str = "W";

    fn cp1_reg(cpu: &Cpu, mut reg: usize) -> Self {
        reg += Cp1::REG_OFFSET;

        if cpu.cp0.is_fr() || (reg & 1) == 0 {
            cpu.regs[reg] as i32
        } else {
            (cpu.regs[reg & !1] >> 32) as i32
        }
    }

    fn set_cp1_reg(cpu: &mut Cpu, mut reg: usize, value: Self) -> RegWrite {
        reg += Cp1::REG_OFFSET;

        let value = if cpu.cp0.is_fr() || (reg & 1) == 0 {
            (cpu.regs[reg] & !0xffff_ffff) | (value as u32 as i64)
        } else {
            reg &= !1;
            (cpu.regs[reg & !1] & 0xffff_ffff) | ((value as u32 as i64) << 32)
        };

        RegWrite { reg, value }
    }

    fn to_f32(self) -> f32 {
        self as _
    }

    fn to_f64(self) -> f64 {
        self as _
    }
}

impl Int for i32 {}

impl Format for i64 {
    const NAME: &'static str = "L";

    fn cp1_reg(cpu: &Cpu, mut reg: usize) -> Self {
        reg += Cp1::REG_OFFSET;

        if cpu.cp0.is_fr() {
            cpu.regs[reg]
        } else {
            cpu.regs[reg & !1]
        }
    }

    fn set_cp1_reg(cpu: &mut Cpu, mut reg: usize, value: Self) -> RegWrite {
        reg += Cp1::REG_OFFSET;

        if !cpu.cp0.is_fr() {
            reg &= !1;
        }

        RegWrite { reg, value }
    }

    fn to_f32(self) -> f32 {
        self as _
    }

    fn to_f64(self) -> f64 {
        self as _
    }
}

impl Int for i64 {}

impl Format for f32 {
    const NAME: &'static str = "S";

    fn cp1_reg(cpu: &Cpu, mut reg: usize) -> Self {
        reg += Cp1::REG_OFFSET;

        Self::from_bits(if cpu.cp0.is_fr() || (reg & 1) == 0 {
            cpu.regs[reg] as u32
        } else {
            (cpu.regs[reg & !1] >> 32) as u32
        })
    }

    fn set_cp1_reg(cpu: &mut Cpu, mut reg: usize, value: Self) -> RegWrite {
        reg += Cp1::REG_OFFSET;

        let value = if cpu.cp0.is_fr() || (reg & 1) == 0 {
            (cpu.regs[reg] & !0xffff_ffff) | (value.to_bits() as i64)
        } else {
            reg &= !1;
            (cpu.regs[reg & !1] & 0xffff_ffff) | ((value.to_bits() as i64) << 32)
        };

        RegWrite { reg, value }
    }

    fn to_f32(self) -> f32 {
        self as _
    }

    fn to_f64(self) -> f64 {
        self as _
    }
}

impl Float for f32 {
    fn round_ties_even(self) -> Self {
        self.round_ties_even()
    }

    fn to_i32(self) -> i32 {
        self as _
    }

    fn to_i64(self) -> i64 {
        self as _
    }
}

impl Format for f64 {
    const NAME: &'static str = "D";

    fn cp1_reg(cpu: &Cpu, mut reg: usize) -> Self {
        reg += Cp1::REG_OFFSET;

        Self::from_bits(if cpu.cp0.is_fr() {
            cpu.regs[reg] as u64
        } else {
            cpu.regs[reg & !1] as u64
        })
    }

    fn set_cp1_reg(cpu: &mut Cpu, mut reg: usize, value: Self) -> RegWrite {
        reg += Cp1::REG_OFFSET;

        if !cpu.cp0.is_fr() {
            reg &= !1;
        }

        RegWrite {
            reg,
            value: value.to_bits() as i64,
        }
    }

    fn to_f32(self) -> f32 {
        self as _
    }

    fn to_f64(self) -> f64 {
        self as _
    }
}

impl Float for f64 {
    fn round_ties_even(self) -> Self {
        self.round_ties_even()
    }

    fn to_i32(self) -> i32 {
        self as _
    }

    fn to_i64(self) -> i64 {
        self as _
    }
}

impl From<RegWrite> for DcOperation {
    fn from(RegWrite { reg, value }: RegWrite) -> Self {
        Self::RegWrite { reg, value }
    }
}
