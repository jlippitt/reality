use super::{Bus, Cpu};
use tracing::trace;

pub fn sync(cpu: &mut Cpu) {
    trace!("{:08X}: SYNC", cpu.pc[0]);
    // This is a NOP on the VR4300
}

pub fn cache(cpu: &mut Cpu, _bus: &mut impl Bus) {
    let base = ((cpu.opcode[0] >> 21) & 31) as usize;
    let op = (cpu.opcode[0] >> 16) & 31;
    let offset = (cpu.opcode[0] & 0xffff) as i16;

    trace!(
        "{:08X}: CACHE 0b{:05b}, {}({})",
        cpu.pc[0],
        op,
        offset,
        Cpu::REG_NAMES[base]
    );

    let vaddr = cpu.regs[base].wrapping_add(offset as i64) as u32;

    let paddr = if vaddr >> 30 == 2 {
        vaddr & 0x1fff_ffff
    } else {
        let Some(result) = cpu.cp0.translate(vaddr) else {
            return;
        };

        result.paddr
    };

    match op {
        0b00000 => {
            let line = cpu.icache.line_mut(vaddr);
            line.clear_valid_flag();
            trace!("ICache Line at {:08X} invalidated", vaddr);
        }
        0b00001 => {
            #[cfg(feature = "dcache")]
            {
                cpu.dcache.index_write_back_invalidate(paddr, |line| {
                    bus.write_block(paddr & 0x1fff_fff0, line.bytes());
                    cpu.stall += super::REFRESH_DCACHE_DELAY;
                    trace!("dcache line at {:08x} written back to memory", paddr)?;
                });
            }
        }
        0b01000 => {
            let tag = &cpu.cp0.tag_lo();
            let ptag = tag.ptag_lo();
            let valid = (tag.pstate() & 0b10) != 0;
            cpu.icache.index_store_tag(vaddr, ptag, valid);
        }
        0b01001 => {
            #[cfg(feature = "dcache")]
            {
                let tag = &cpu.cp0.tag_lo();
                let ptag = tag.ptag_lo();
                let valid = (tag.pstate() & 0b10) != 0;
                let dirty = (tag.pstate() & 0b01) != 0;
                cpu.dcache.index_store_tag(paddr, ptag, valid, dirty);
            }
        }
        0b01101 => {
            #[cfg(feature = "dcache")]
            {
                cpu.dcache.create_dirty_exclusive(paddr, |line| {
                    bus.write_block(paddr & 0x1fff_fff0, line.bytes());
                    cpu.stall += super::REFRESH_DCACHE_DELAY;
                    trace!("DCache Line at {:08X} written back to memory", paddr)?;
                });
            }
        }
        0b10000 => {
            if let Some(line) = cpu.icache.find_mut(vaddr, paddr) {
                line.clear_valid_flag();
                trace!("ICache Line at {:08X} invalidated", vaddr);
            }
        }
        0b10001 =>
        {
            #[cfg(feature = "dcache")]
            if let Some(line) = cpu.dcache.find_mut(paddr)? {
                line.clear_valid_flag();
                trace!("DCache Line at {:08X} invalidated", paddr)?;
            }
        }
        0b10101 => {
            #[cfg(feature = "dcache")]
            {
                cpu.dcache.hit_write_back_invalidate(paddr, |line| {
                    bus.write_block(paddr & 0x1fff_fff0, line.bytes());
                    cpu.stall += super::REFRESH_DCACHE_DELAY;
                    trace!("DCache Line at {:08X} written back to memory", paddr)?;
                });
            }
        }
        0b11001 =>
        {
            #[cfg(feature = "dcache")]
            if let Some(line) = cpu.dcache.find_mut(paddr)? {
                if line.is_dirty() {
                    bus.write_block(paddr & 0x1fff_fff0, line.bytes());
                    cpu.stall += super::REFRESH_DCACHE_DELAY;
                    line.clear_dirty_flag();
                    trace!("DCache Line at {:08X} written back to memory", paddr)?;
                }
            }
        }
        op => todo!("Cache Operation: {:05b}", op),
    }
}
