use super::{Cpu, DcState};
use tracing::trace;

pub fn tlbwi(cpu: &mut Cpu, pc: u32) -> DcState {
    trace!("{:08X}: TLBWI", pc);
    cpu.cp0.tlb.write_entry(&cpu.cp0.regs);
    DcState::Nop
}

pub fn tlbp(cpu: &mut Cpu, pc: u32) -> DcState {
    trace!("{:08X}: TLBP", pc);

    let regs = &mut cpu.cp0.regs;

    let index = cpu.cp0.tlb.entries().position(|entry| {
        entry.entry_hi.vpn2() == regs.entry_hi.vpn2()
            && (entry.entry_hi.global() || (entry.entry_hi.asid() == regs.entry_hi.asid()))
    });

    if let Some(index) = index {
        regs.index.set_index(index as u32);
        regs.index.set_probe_failure(false);
    } else {
        regs.index.set_probe_failure(true);
    }

    trace!("  Index: {:?}", regs.index);

    DcState::Nop
}
