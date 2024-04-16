use super::{Cpu, DcOperation, Float, Format};
use tracing::trace;

pub fn cvt_s<F: Format>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.S.{} F{}, F{}", pc, F::NAME, fd, fs);
    f32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_f32()).into()
}

pub fn cvt_d<F: Format>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.D.{} F{}, F{}", pc, F::NAME, fd, fs);
    f64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_f64()).into()
}

pub fn cvt_w<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.W.{} F{}, F{}", pc, F::NAME, fd, fs);
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_i32()).into()
}

pub fn cvt_l<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CVT.L.{} F{}, F{}", pc, F::NAME, fd, fs);
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_i64()).into()
}

pub fn round_w<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: ROUND.W.{} F{}, F{}", pc, F::NAME, fd, fs);
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).round_ties_even().to_i32()).into()
}

pub fn round_l<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: ROUND.L:.{} F{}, F{}", pc, F::NAME, fd, fs);
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).round_ties_even().to_i64()).into()
}

pub fn trunc_w<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: TRUNC.W.{} F{}, F{}", pc, F::NAME, fd, fs);
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).trunc().to_i32()).into()
}

pub fn trunc_l<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: TRUNC.L:.{} F{}, F{}", pc, F::NAME, fd, fs);
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).trunc().to_i64()).into()
}

pub fn ceil_w<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CEIL.W.{} F{}, F{}", pc, F::NAME, fd, fs);
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).ceil().to_i32()).into()
}

pub fn ceil_l<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: CEIL.L:.{} F{}, F{}", pc, F::NAME, fd, fs);
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).ceil().to_i64()).into()
}

pub fn floor_w<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: FLOOR.W.{} F{}, F{}", pc, F::NAME, fd, fs);
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).floor().to_i32()).into()
}

pub fn floor_l<F: Float>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let fs = ((word >> 11) & 31) as usize;
    let fd = ((word >> 6) & 31) as usize;
    trace!("{:08X}: FLOOR.L:.{} F{}, F{}", pc, F::NAME, fd, fs);
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).floor().to_i64()).into()
}
