pub use transfer::{Ldc1, Lwc1, Sdc1, Swc1};

use super::load::LoadOperator;
use super::store::StoreOperator;
use super::{Cpu, DcState, Float, Format, Int};

mod convert;
mod transfer;

pub fn cop1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match (word >> 21) & 31 {
        0o00 => transfer::mfc1(cpu, pc, word),
        0o01 => transfer::dmfc1(cpu, pc, word),
        0o04 => transfer::mtc1(cpu, pc, word),
        0o05 => transfer::dmtc1(cpu, pc, word),
        0o20 => float::<f32>(cpu, pc, word),
        0o21 => float::<f64>(cpu, pc, word),
        0o24 => int::<i32>(cpu, pc, word),
        0o25 => int::<i64>(cpu, pc, word),
        opcode => todo!("CPU COP1 Opcode '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn float<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match word & 63 {
        0o40 => convert::cvt_s::<F>(cpu, pc, word),
        0o41 => convert::cvt_d::<F>(cpu, pc, word),
        0o44 => convert::cvt_w::<F>(cpu, pc, word),
        0o45 => convert::cvt_l::<F>(cpu, pc, word),
        opcode => todo!("CPU COP1 Float Function '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn int<F: Int>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match word & 63 {
        0o40 => convert::cvt_s::<F>(cpu, pc, word),
        0o41 => convert::cvt_d::<F>(cpu, pc, word),
        opcode => todo!("CPU COP1 Int Function '{:02o}' at {:08X}", opcode, pc),
    }
}
