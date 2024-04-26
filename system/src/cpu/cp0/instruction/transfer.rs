use super::Cp0;
use super::Cpu;
use tracing::trace;

pub fn mfc0(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: MFC0 {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    let value = cpu.cp0.read_reg(rd) as i32 as i64;
    cpu.set_reg(rt, value);
}

pub fn dmfc0(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: DMFC0 {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    let value = cpu.cp0.read_reg(rd);
    cpu.set_reg(rt, value);
}

pub fn mtc0(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: MTC0 {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    cpu.cp0.write_reg(rd, cpu.regs[rt] as i32 as i64);
}

pub fn dmtc0(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: DMTC0 {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cp0::REG_NAMES[rd]
    );

    cpu.cp0.write_reg(rd, cpu.regs[rt]);
}
