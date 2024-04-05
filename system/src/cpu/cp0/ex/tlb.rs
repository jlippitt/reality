use super::{Cpu, DcState};
use tracing::trace;

pub fn tlbwi(cpu: &mut Cpu, pc: u32) -> DcState {
    trace!("{:08X}: TLBWI", pc);
    cpu.cp0.tlb.write_entry(&cpu.cp0.regs);
    DcState::Nop
}
