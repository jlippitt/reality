pub use ex::{cop1, ldc1, lwc1, sdc1, swc1};

use super::{Cpu, DcState};
use bytemuck::Pod;
use regs::Status;

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
    _status: Status,
}

impl Cp1 {
    pub const REG_OFFSET: usize = 32;

    pub fn new() -> Self {
        Self {
            _status: Status::default(),
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

impl From<RegWrite> for DcState {
    fn from(RegWrite { reg, value }: RegWrite) -> Self {
        Self::RegWrite { reg, value }
    }
}
