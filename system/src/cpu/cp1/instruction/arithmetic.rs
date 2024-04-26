use super::{Cpu, Float};
use tracing::trace;

pub fn add<F: Float>(cpu: &mut Cpu) {
    let ft = ((cpu.opcode[0] >> 16) & 31) as usize;
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: ADD.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 2;
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) + F::cp1_reg(cpu, ft))
}

pub fn sub<F: Float>(cpu: &mut Cpu) {
    let ft = ((cpu.opcode[0] >> 16) & 31) as usize;
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: SUB.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 2;
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) - F::cp1_reg(cpu, ft))
}

pub fn mul<F: Float>(cpu: &mut Cpu) {
    let ft = ((cpu.opcode[0] >> 16) & 31) as usize;
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: MUL.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    // TODO: Double this if using 'D' format
    cpu.stall += 5;
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) * F::cp1_reg(cpu, ft))
}

pub fn div<F: Float>(cpu: &mut Cpu) {
    let ft = ((cpu.opcode[0] >> 16) & 31) as usize;
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: DIV.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    // TODO: Double this if using 'D' format
    cpu.stall += 29;
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs) / F::cp1_reg(cpu, ft))
}

pub fn sqrt<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: SQRT.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    cpu.stall += 29;
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).sqrt())
}

pub fn abs<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: ABS.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs).abs())
}

pub fn mov<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: MOV.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, F::cp1_reg(cpu, fs))
}

pub fn neg<F: Float>(cpu: &mut Cpu) {
    let fs = ((cpu.opcode[0] >> 11) & 31) as usize;
    let fd = ((cpu.opcode[0] >> 6) & 31) as usize;
    trace!("{:08X}: NEG.{} F{}, F{}", cpu.pc[0], F::NAME, fd, fs);
    F::set_cp1_reg(cpu, fd, -F::cp1_reg(cpu, fs))
}
