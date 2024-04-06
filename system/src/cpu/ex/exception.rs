use super::{Cpu, DcState};
use tracing::trace;

pub fn teq(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;

    trace!(
        "{:08X}: TEQ {}, {}",
        pc,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    if cpu.regs[rs] == cpu.regs[rt] {
        todo!("TrapException");
    }

    DcState::Nop
}

pub fn tne(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;

    trace!(
        "{:08X}: TNE {}, {}",
        pc,
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    if cpu.regs[rs] != cpu.regs[rt] {
        todo!("TrapException");
    }

    DcState::Nop
}
