pub use transfer::{ldc1, lwc1, sdc1, swc1};

use super::{Cp1, Cpu, DcOperation, Float, Format, Int};

mod arithmetic;
mod branch;
mod compare;
mod convert;
mod transfer;

pub fn cop1(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    match (word >> 21) & 31 {
        0o00 => transfer::mfc1(cpu, pc, word),
        0o01 => transfer::dmfc1(cpu, pc, word),
        0o02 => transfer::cfc1(cpu, pc, word),
        0o04 => transfer::mtc1(cpu, pc, word),
        0o05 => transfer::dmtc1(cpu, pc, word),
        0o06 => transfer::ctc1(cpu, pc, word),
        0o10 => bc(cpu, pc, word),
        0o20 => float::<f32>(cpu, pc, word),
        0o21 => float::<f64>(cpu, pc, word),
        0o24 => int::<i32>(cpu, pc, word),
        0o25 => int::<i64>(cpu, pc, word),
        opcode => todo!("CPU COP1 Opcode '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn bc(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    match (word >> 16) & 31 {
        0o00 => branch::bc1f::<false>(cpu, pc, word),
        0o01 => branch::bc1t::<false>(cpu, pc, word),
        0o02 => branch::bc1f::<true>(cpu, pc, word),
        0o03 => branch::bc1t::<true>(cpu, pc, word),
        opcode => todo!("CPU COP1 BC Function '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn float<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    match word & 63 {
        0o00 => arithmetic::add::<F>(cpu, pc, word),
        0o01 => arithmetic::sub::<F>(cpu, pc, word),
        0o02 => arithmetic::mul::<F>(cpu, pc, word),
        0o03 => arithmetic::div::<F>(cpu, pc, word),
        0o04 => arithmetic::sqrt::<F>(cpu, pc, word),
        0o05 => arithmetic::abs::<F>(cpu, pc, word),
        0o06 => arithmetic::mov::<F>(cpu, pc, word),
        0o07 => arithmetic::neg::<F>(cpu, pc, word),
        0o10 => convert::round_l::<F>(cpu, pc, word),
        0o11 => convert::trunc_l::<F>(cpu, pc, word),
        0o12 => convert::ceil_l::<F>(cpu, pc, word),
        0o13 => convert::floor_l::<F>(cpu, pc, word),
        0o14 => convert::round_w::<F>(cpu, pc, word),
        0o15 => convert::trunc_w::<F>(cpu, pc, word),
        0o16 => convert::ceil_w::<F>(cpu, pc, word),
        0o17 => convert::floor_w::<F>(cpu, pc, word),
        0o40 => convert::cvt_s::<F>(cpu, pc, word),
        0o41 => convert::cvt_d::<F>(cpu, pc, word),
        0o44 => convert::cvt_w::<F>(cpu, pc, word),
        0o45 => convert::cvt_l::<F>(cpu, pc, word),
        0o60 => compare::c::<compare::F, F>(cpu, pc, word),
        0o61 => compare::c::<compare::UN, F>(cpu, pc, word),
        0o62 => compare::c::<compare::EQ, F>(cpu, pc, word),
        0o63 => compare::c::<compare::UEQ, F>(cpu, pc, word),
        0o64 => compare::c::<compare::OLT, F>(cpu, pc, word),
        0o65 => compare::c::<compare::ULT, F>(cpu, pc, word),
        0o66 => compare::c::<compare::OLE, F>(cpu, pc, word),
        0o67 => compare::c::<compare::ULE, F>(cpu, pc, word),
        0o70 => compare::c::<compare::SF, F>(cpu, pc, word),
        0o71 => compare::c::<compare::NGLE, F>(cpu, pc, word),
        0o72 => compare::c::<compare::SEQ, F>(cpu, pc, word),
        0o73 => compare::c::<compare::NGL, F>(cpu, pc, word),
        0o74 => compare::c::<compare::LT, F>(cpu, pc, word),
        0o75 => compare::c::<compare::NGE, F>(cpu, pc, word),
        0o76 => compare::c::<compare::LE, F>(cpu, pc, word),
        0o77 => compare::c::<compare::NGT, F>(cpu, pc, word),
        opcode => todo!("CPU COP1 Float Function '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn int<F: Int>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    match word & 63 {
        0o40 => convert::cvt_s::<F>(cpu, pc, word),
        0o41 => convert::cvt_d::<F>(cpu, pc, word),
        opcode => todo!("CPU COP1 Int Function '{:02o}' at {:08X}", opcode, pc),
    }
}
