use super::{Cpu, DcOperation, Float};
use tracing::trace;

pub fn add<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let ft = ((word >> 16) & 31) as usize;
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: ADD.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) + F::cp1_reg(cpu, ft)).into()
}

pub fn sub<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let ft = ((word >> 16) & 31) as usize;
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: SUB.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) - F::cp1_reg(cpu, ft)).into()
}

pub fn mul<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let ft = ((word >> 16) & 31) as usize;
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: MUL.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) * F::cp1_reg(cpu, ft)).into()
}

pub fn div<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let ft = ((word >> 16) & 31) as usize;
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: DIV.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) / F::cp1_reg(cpu, ft)).into()
}

pub fn sqrt<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: SQRT.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).sqrt()).into()
}

pub fn abs<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: ABS.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).abs()).into()
}

pub fn mov<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: MOV.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs)).into()
}

pub fn neg<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: NEG.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, -F::cp1_reg(cpu, fs)).into()
}
