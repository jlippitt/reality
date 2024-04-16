use super::Cp0;
use super::{Cpu, DcOperation};
use tracing::trace;

mod tlb;
mod transfer;

pub fn cop0(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    match (word >> 21) & 31 {
        0o00 => transfer::mfc0(cpu, pc, word),
        0o01 => transfer::dmfc0(cpu, pc, word),
        0o04 => transfer::mtc0(cpu, pc, word),
        0o05 => transfer::dmtc0(cpu, pc, word),
        0o20..=0o37 => match word & 63 {
            0o01 => tlb::tlbr(cpu, pc),
            0o02 => tlb::tlbwi(cpu, pc),
            0o10 => tlb::tlbp(cpu, pc),
            0o30 => eret(cpu, pc),
            func => todo!("CPU COP0 Function '{:02o}' at {:08X}", func, pc),
        },
        opcode => todo!("CPU COP0 Opcode '{:02o}' at {:08X}", opcode, pc),
    }
}

fn eret(cpu: &mut Cpu, pc: u32) -> DcOperation {
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

    // Prevent re-running ERET after exception
    // (Should this be written in a later pipeline stage?)
    cpu.ex.pc = cpu.pc;
    cpu.ex.word = 0;

    // The delay slot instruction is not executed
    cpu.rf.pc = cpu.pc;
    cpu.rf.word = 0;

    DcOperation::Nop
}
