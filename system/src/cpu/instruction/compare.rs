use super::Cpu;
use tracing::trace;

pub fn slti(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let imm = (cpu.opcode[0] & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: SLTI {}, {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    cpu.set_reg(rt, (cpu.regs[rs] < imm) as i64);
}

pub fn sltiu(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let imm = (cpu.opcode[0] & 0xffff) as i16 as u64;

    trace!(
        "{:08X}: SLTIU {}, {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    cpu.set_reg(rt, ((cpu.regs[rs] as u64) < imm) as i64);
}

pub fn slt(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLT {}, {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    cpu.set_reg(rd, (cpu.regs[rs] < cpu.regs[rt]) as i64);
}

pub fn sltu(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLTU {}, {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    cpu.set_reg(rd, ((cpu.regs[rs] as u64) < (cpu.regs[rt] as u64)) as i64);
}
