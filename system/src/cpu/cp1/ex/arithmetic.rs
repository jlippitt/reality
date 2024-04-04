use super::{Cpu, DcState, Float};
use tracing::trace;

pub fn sqrt<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: SQRT.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).sqrt()).into()
}

pub fn abs<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: ABS.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).abs()).into()
}

pub fn mov<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: MOV.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs)).into()
}

pub fn neg<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: NEG.{} F{}, F{}", pc, F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, -F::cp1_reg(cpu, fs)).into()
}
