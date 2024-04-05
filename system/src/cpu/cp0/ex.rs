use super::Cp0;
use super::{Cpu, DcState};
use tracing::trace;

mod tlb;
mod transfer;

pub fn cop0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match (word >> 21) & 31 {
        0o00 => transfer::mfc0(cpu, pc, word),
        0o04 => transfer::mtc0(cpu, pc, word),
        0o20..=0o37 => match word & 63 {
            0o02 => tlb::tlbwi(cpu, pc),
            0o30 => eret(cpu, pc),
            func => todo!("CPU COP0 Function '{:02o}' at {:08X}", func, pc),
        },
        opcode => todo!("CPU COP0 Opcode '{:02o}' at {:08X}", opcode, pc),
    }
}

fn eret(cpu: &mut Cpu, pc: u32) -> DcState {
    trace!("{:08X}: ERET", pc);

    let regs = &mut cpu.cp0.regs;

    if regs.status.erl() {
        cpu.pc = regs.error_epc;
        regs.status.set_erl(false);
    } else {
        cpu.pc = regs.epc;
        regs.status.set_exl(false);
    }

    cpu.ll_bit = false;
    cpu.rf.word = 0;

    DcState::Nop
}
