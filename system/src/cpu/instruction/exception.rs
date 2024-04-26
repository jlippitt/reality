use super::cp0;
use super::Cpu;
use tracing::trace;

pub fn cop2(cpu: &mut Cpu) {
    if cpu.cp0.cp2_usable() {
        match (cpu.opcode[0] >> 21) & 31 {
            0o00 | 0x01 | 0x02 | 0x04 | 0x05 | 0x06 => (),
            _ => cp0::except(cpu, cp0::Exception::ReservedInstruction(2)),
        }
    } else {
        cp0::except(cpu, cp0::Exception::CoprocessorUnusable(2));
    }
}

pub fn syscall(cpu: &mut Cpu) {
    trace!("{:08X}: SYSCALL", cpu.pc[0]);
    cp0::except(cpu, cp0::Exception::Syscall);
}

pub fn break_(cpu: &mut Cpu) {
    trace!("{:08X}: BREAK", cpu.pc[0]);
    cp0::except(cpu, cp0::Exception::Breakpoint);
}

pub fn teq(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;

    trace!(
        "{:08X}: TEQ {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    if cpu.regs[rs] == cpu.regs[rt] {
        todo!("TrapException");
    }
}

pub fn tne(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;

    trace!(
        "{:08X}: TNE {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    if cpu.regs[rs] != cpu.regs[rt] {
        todo!("TrapException");
    }
}
