use super::{Cpu, DcState, Float, Format};
use tracing::trace;

pub fn cvt_s<F: Format>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.S.{} F{}, F{}", pc, F::NAME, fd, fs);
    f32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_f32()).into()
}

pub fn cvt_d<F: Format>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.D.{} F{}, F{}", pc, F::NAME, fd, fs);
    f64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_f64()).into()
}

pub fn cvt_w<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.W.{} F{}, F{}", pc, F::NAME, fd, fs);
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_i32()).into()
}

pub fn cvt_l<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.L.{} F{}, F{}", pc, F::NAME, fd, fs);
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_i64()).into()
}
