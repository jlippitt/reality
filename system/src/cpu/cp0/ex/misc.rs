use super::{Cpu, DcState};
use tracing::trace;

pub fn cache(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    const CACHE_OP_NAMES: [char; 8] = ['?', '?', 'P', '?', '?', '?', '?', '?'];
    const CACHE_NAMES: [char; 4] = ['I', 'D', '?', '?'];

    let base = ((word >> 21) & 31) as usize;
    let op = (word >> 16) & 31;
    let offset = (word & 0xffff) as i16;

    trace!(
        "{:08X}: CACHE {}{}, {}({})",
        pc,
        CACHE_OP_NAMES[(op >> 2) as usize],
        CACHE_NAMES[(op & 3) as usize],
        offset,
        Cpu::REG_NAMES[base]
    );

    let address = cpu.regs[base].wrapping_add(offset as i64) as u32;

    match (word >> 16) & 31 {
        0b01000 => {
            let tag = &cpu.cp0.regs.tag_lo;
            let ptag = tag.ptag_lo();
            let valid = (tag.pstate() & 0b10) != 0;
            cpu.icache.index_store_tag(address, ptag, valid);
            DcState::Nop
        }
        0b01001 => {
            let tag = &cpu.cp0.regs.tag_lo;
            let ptag = tag.ptag_lo();
            let valid = (tag.pstate() & 0b10) != 0;
            let dirty = (tag.pstate() & 0b01) != 0;
            cpu.dcache.index_store_tag(address, ptag, valid, dirty);
            DcState::Nop
        }
        0b10000 => {
            if let Some(line) = cpu.icache.find_mut(address) {
                line.clear_valid_flag();
                trace!("ICache Line at {:08X} invalidated", address);
            }
            DcState::Nop
        }
        0b10001 => {
            if let Some(line) = cpu.dcache.find_mut(address) {
                line.clear_valid_flag();
                trace!("DCache Line at {:08X} invalidated", address);
            }
            DcState::Nop
        }
        0b11001 => DcState::DCacheWriteBack { addr: address },
        op => todo!("Cache Operation: {:05b}", op),
    }
}

pub fn eret(cpu: &mut Cpu, pc: u32) -> DcState {
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