use super::regs::EntryHi;
use super::Cpu;
use tracing::trace;

pub fn tlbr(cpu: &mut Cpu) {
    trace!("{:08X}: TLBR", cpu.pc[0]);
    let index = cpu.cp0.regs.index.index() as usize;
    cpu.cp0.tlb.read_entry(&mut cpu.cp0.regs, index);
}

pub fn tlbwi(cpu: &mut Cpu) {
    trace!("{:08X}: TLBWI", cpu.pc[0]);
    let index = cpu.cp0.regs.index.index() as usize;
    cpu.cp0.tlb.write_entry(&cpu.cp0.regs, index);
}

pub fn tlbwr(cpu: &mut Cpu) {
    trace!("{:08X}: TLBWR", cpu.pc[0]);
    let index = cpu.cp0.regs.random as usize;
    cpu.cp0.tlb.write_entry(&cpu.cp0.regs, index);
}

pub fn tlbp(cpu: &mut Cpu) {
    trace!("{:08X}: TLBP", cpu.pc[0]);

    let regs = &mut cpu.cp0.regs;

    let index = cpu.cp0.tlb.entries().position(|entry| {
        let entry_hi =
            EntryHi::from(u64::from(regs.entry_hi) & !(u32::from(entry.page_mask) as u64));

        entry.entry_hi.vpn2() == entry_hi.vpn2()
            && entry.entry_hi.region() == entry_hi.region()
            && (entry.entry_hi.global() || (entry.entry_hi.asid() == entry_hi.asid()))
    });

    if let Some(index) = index {
        regs.index.set_index(index as u32);
        regs.index.set_probe_failure(false);
    } else {
        regs.index.set_index(0);
        regs.index.set_probe_failure(true);
    }

    trace!("  Index: {:?}", regs.index);
}
