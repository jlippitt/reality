use super::{Cpu, Float, Format};
use tracing::trace;

pub fn cvt_s<F: Format>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: CVT.S.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    // TODO: Fewer cycles if source format is D
    cpu.stall += 5;
    f32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_f32())
}

pub fn cvt_d<F: Format>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: CVT.D.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    // TODO: Fewer cycles if source format is S
    cpu.stall += 5;
    f64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_f64())
}

pub fn cvt_w<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: CVT.W.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_i32())
}

pub fn cvt_l<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: CVT.L.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).to_i64())
}

pub fn round_w<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: ROUND.W.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).round_ties_even().to_i32())
}

pub fn round_l<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: ROUND.L:.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).round_ties_even().to_i64())
}

pub fn trunc_w<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: TRUNC.W.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).trunc().to_i32())
}

pub fn trunc_l<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: TRUNC.L:.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).trunc().to_i64())
}

pub fn ceil_w<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: CEIL.W.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).ceil().to_i32())
}

pub fn ceil_l<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: CEIL.L:.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).ceil().to_i64())
}

pub fn floor_w<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: FLOOR.W.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i32::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).floor().to_i32())
}

pub fn floor_l<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: FLOOR.L:.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 5;
    i64::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).floor().to_i64())
}
