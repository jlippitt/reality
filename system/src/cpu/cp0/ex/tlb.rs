use super::regs::EntryHi;
use super::{Cpu, DcOperation};
use tracing::trace;

pub fn tlbr(cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: TLBR", pc);
    let index = cpu.cp0.regs.index.index() as usize;
    cpu.cp0.tlb.read_entry(&mut cpu.cp0.regs, index);
    DcOperation::Nop
}

pub fn tlbwi(cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: TLBWI", pc);
    cpu.cp0.tlb.write_entry(&cpu.cp0.regs);
    DcOperation::Nop
}

pub fn tlbp(cpu: &mut Cpu, pc: u32) -> DcOperation {
    trace!("{:08X}: TLBP", pc);

    let regs = &mut cpu.cp0.regs;

    let index = cpu.cp0.tlb.entries().position(|entry| {
        let entry_hi = EntryHi::from(u32::from(regs.entry_hi) & !u32::from(entry.page_mask));
        entry.entry_hi.vpn2() == entry_hi.vpn2()
            && (entry.entry_hi.global() || (entry.entry_hi.asid() == entry_hi.asid()))
    });

    if let Some(index) = index {
        regs.index.set_index(index as u32);
        regs.index.set_probe_failure(false);
    } else {
        regs.index.set_probe_failure(true);
    }

    trace!("  Index: {:?}", regs.index);

    DcOperation::Nop
}
