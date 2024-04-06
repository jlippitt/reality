use super::cp1::Format;
use super::{Bus, Cp0, Cpu, WbOperation};
use tracing::trace;

#[derive(Debug)]
pub enum DcState {
    RegWrite { reg: usize, value: i64 },
    LoadByte { reg: usize, addr: u32 },
    LoadByteUnsigned { reg: usize, addr: u32 },
    LoadHalfword { reg: usize, addr: u32 },
    LoadHalfwordUnsigned { reg: usize, addr: u32 },
    LoadWord { reg: usize, addr: u32 },
    LoadWordUnsigned { reg: usize, addr: u32 },
    LoadWordLeft { reg: usize, addr: u32 },
    LoadWordRight { reg: usize, addr: u32 },
    LoadDoubleword { reg: usize, addr: u32 },
    LoadDoublewordLeft { reg: usize, addr: u32 },
    LoadDoublewordRight { reg: usize, addr: u32 },
    LoadLinked { reg: usize, addr: u32 },
    LoadLinkedDoubleword { reg: usize, addr: u32 },
    StoreByte { value: u8, addr: u32 },
    StoreHalfword { value: u16, addr: u32 },
    StoreWord { value: u32, addr: u32 },
    StoreWordLeft { value: u32, addr: u32 },
    StoreWordRight { value: u32, addr: u32 },
    StoreDoubleword { value: u64, addr: u32 },
    StoreDoublewordLeft { value: u64, addr: u32 },
    StoreDoublewordRight { value: u64, addr: u32 },
    StoreConditional { reg: usize, value: u32, addr: u32 },
    StoreConditionalDoubleword { reg: usize, value: u64, addr: u32 },
    Cp0RegWrite { reg: usize, value: i64 },
    Cp1ControlRegWrite { reg: usize, value: u32 },
    Cp1LoadWord { reg: usize, addr: u32 },
    Cp1LoadDoubleword { reg: usize, addr: u32 },
    CacheOperation { op: u32, vaddr: u32 },
    Nop,
}

pub fn execute(cpu: &mut Cpu, bus: &mut impl Bus) {
    match cpu.dc {
        DcState::RegWrite { reg, value } => {
            cpu.wb.reg = reg;
            cpu.wb.value = value;
            cpu.wb.op = None;
        }
        DcState::LoadByte { reg, addr } => {
            // TODO: Stall cycles
            let value = cpu.read::<u8>(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i8 as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DcState::LoadByteUnsigned { reg, addr } => {
            // TODO: Stall cycles
            let value = cpu.read::<u8>(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DcState::LoadHalfword { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 1) == 0);
            let value = cpu.read::<u16>(bus, addr) as i16 as i64;
            cpu.wb.reg = reg;
            cpu.wb.value = value as i16 as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DcState::LoadHalfwordUnsigned { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 1) == 0);
            let value = cpu.read::<u16>(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DcState::LoadWord { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            let value = cpu.read::<u32>(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32 as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DcState::LoadWordLeft { reg, addr } => {
            // TODO: Stall cycles
            let value = cpu.read::<u32>(bus, addr & !3);
            let shift = (addr & 3) << 3;
            cpu.wb.reg = reg;
            cpu.wb.value =
                (cpu.regs[reg] as u32 & !(u32::MAX << shift) | (value << shift)) as i32 as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DcState::LoadWordRight { reg, addr } => {
            // TODO: Stall cycles
            let value = cpu.read::<u32>(bus, addr & !3);
            let shift = (addr & 3 ^ 3) << 3;
            cpu.wb.reg = reg;
            cpu.wb.value =
                (cpu.regs[reg] as u32 & !(u32::MAX >> shift) | (value >> shift)) as i32 as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DcState::LoadWordUnsigned { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            let value = cpu.read::<u32>(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DcState::LoadDoubleword { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 7) == 0);
            let value = cpu.read_dword(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:016X}]", addr, value);
        }
        DcState::LoadDoublewordLeft { reg, addr } => {
            // TODO: Stall cycles
            let value = cpu.read_dword(bus, addr & !7);
            let shift = (addr & 7) << 3;
            cpu.wb.reg = reg;
            cpu.wb.value = (cpu.regs[reg] as u64 & !(u64::MAX << shift) | (value << shift)) as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:016X}]", addr, value);
        }
        DcState::LoadDoublewordRight { reg, addr } => {
            // TODO: Stall cycles
            let value = cpu.read_dword(bus, addr & !7);
            let shift = (addr & 7 ^ 7) << 3;
            cpu.wb.reg = reg;
            cpu.wb.value = (cpu.regs[reg] as u64 & !(u64::MAX >> shift) | (value >> shift)) as i64;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:016X}]", addr, value);
        }
        DcState::LoadLinked { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            let value = cpu.read::<u32>(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32 as i64;
            trace!("  [{:08X} => {:08X}]", addr, value);

            // LLAddr is set to physical address
            // TODO: Remove this hack when TLB support is implemented
            cpu.wb.op = Some(WbOperation::Cp0RegWrite {
                reg: Cp0::LL_ADDR,
                value: ((addr & 0x1fff_ffff) >> 4) as i64,
            });
            cpu.ll_bit = true;
        }
        DcState::LoadLinkedDoubleword { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 7) == 0);
            let value = cpu.read_dword(bus, addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i64;
            trace!("  [{:08X} => {:016X}]", addr, value);

            // LLAddr is set to physical address
            // TODO: Remove this hack when TLB support is implemented
            cpu.wb.op = Some(WbOperation::Cp0RegWrite {
                reg: Cp0::LL_ADDR,
                value: ((addr & 0x1fff_ffff) >> 4) as i64,
            });
            cpu.ll_bit = true;
        }
        DcState::StoreByte { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:02X}]", addr, value);
            cpu.write(bus, addr, value);
        }
        DcState::StoreHalfword { value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 1) == 0);
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:04X}]", addr, value);
            cpu.write(bus, addr, value);
        }
        DcState::StoreWord { value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:08X}]", addr, value);
            cpu.write(bus, addr, value);
        }
        DcState::StoreWordLeft { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:08X}]", addr, value);

            match addr & 3 {
                0 => cpu.write(bus, addr & !3, value),
                1 => {
                    cpu.write(bus, addr & !3 | 1, (value >> 24) as u8);
                    cpu.write(bus, addr & !3 | 2, (value >> 8) as u16);
                }
                2 => cpu.write(bus, addr & !3 | 2, (value >> 16) as u16),
                _ => cpu.write(bus, addr & !3 | 3, (value >> 24) as u8),
            }
        }
        DcState::StoreWordRight { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:08X}]", addr, value);

            match addr & 3 {
                0 => cpu.write(bus, addr & !3, value as u8),
                1 => cpu.write(bus, addr & !3, value as u16),
                2 => {
                    cpu.write(bus, addr & !3, (value >> 8) as u16);
                    cpu.write(bus, addr & !3 | 2, value as u8);
                }
                _ => cpu.write(bus, addr & !3, value),
            }
        }
        DcState::StoreDoubleword { value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 7) == 0);
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:016X}]", addr, value);
            cpu.write_dword(bus, addr, value);
        }
        DcState::StoreDoublewordLeft { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:08X}]", addr, value);

            match addr & 7 {
                0 => cpu.write_dword(bus, addr & !7, value),
                1 => {
                    cpu.write(bus, addr & !7 | 1, (value >> 56) as u8);
                    cpu.write(bus, addr & !7 | 2, (value >> 40) as u16);
                    cpu.write(bus, addr & !7 | 4, (value >> 8) as u32);
                }
                2 => {
                    cpu.write(bus, addr & !7 | 2, (value >> 48) as u16);
                    cpu.write(bus, addr & !7 | 4, (value >> 16) as u32);
                }
                3 => {
                    cpu.write(bus, addr & !7 | 3, (value >> 56) as u8);
                    cpu.write(bus, addr & !7 | 4, (value >> 24) as u32);
                }
                4 => cpu.write(bus, addr & !7 | 4, (value >> 32) as u32),
                5 => {
                    cpu.write(bus, addr & !7 | 5, (value >> 56) as u8);
                    cpu.write(bus, addr & !7 | 6, (value >> 40) as u16);
                }
                6 => cpu.write(bus, addr & !7 | 6, (value >> 48) as u16),
                _ => cpu.write(bus, addr & !7 | 7, (value >> 56) as u8),
            }
        }
        DcState::StoreDoublewordRight { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:08X}]", addr, value);

            match addr & 7 {
                0 => cpu.write(bus, addr & !7, value as u8),
                1 => cpu.write(bus, addr & !7, value as u16),
                2 => {
                    cpu.write(bus, addr & !7, (value >> 8) as u16);
                    cpu.write(bus, addr & !7 | 2, value as u8);
                }
                3 => cpu.write(bus, addr & !7, value as u32),
                4 => {
                    cpu.write(bus, addr & !7, (value >> 8) as u32);
                    cpu.write(bus, addr & !7 | 4, value as u8);
                }
                5 => {
                    cpu.write(bus, addr & !7, (value >> 16) as u32);
                    cpu.write(bus, addr & !7 | 4, value as u16);
                }
                6 => {
                    cpu.write(bus, addr & !7, (value >> 24) as u32);
                    cpu.write(bus, addr & !7 | 4, (value >> 8) as u16);
                    cpu.write(bus, addr & !7 | 6, value as u8);
                }
                _ => cpu.write_dword(bus, addr & !7, value),
            }
        }
        DcState::StoreConditional { reg, value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            let ll_bit = cpu.ll_bit;
            cpu.wb.reg = reg;
            cpu.wb.value = ll_bit as i64;
            cpu.wb.op = None;

            if ll_bit {
                trace!("  [{:08X} <= {:08X}]", addr, value);
                cpu.write(bus, addr, value);
            }
        }
        DcState::StoreConditionalDoubleword { reg, value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 7) == 0);
            let ll_bit = cpu.ll_bit;
            cpu.wb.reg = reg;
            cpu.wb.value = ll_bit as i64;
            cpu.wb.op = None;

            if ll_bit {
                trace!("  [{:08X} <= {:016X}]", addr, value);
                cpu.write_dword(bus, addr, value);
            }
        }
        DcState::Cp0RegWrite { reg, value } => {
            cpu.wb.reg = 0;
            cpu.wb.op = Some(WbOperation::Cp0RegWrite { reg, value });
        }
        DcState::Cp1ControlRegWrite { reg, value } => {
            cpu.wb.reg = 0;
            cpu.wb.op = Some(WbOperation::Cp1ControlRegWrite { reg, value });
        }
        DcState::Cp1LoadWord { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            let value = cpu.read::<u32>(bus, addr);
            let reg_write = i32::set_cp1_reg(cpu, reg, value as i32);
            cpu.wb.reg = reg_write.reg;
            cpu.wb.value = reg_write.value;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DcState::Cp1LoadDoubleword { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 7) == 0);
            let value = cpu.read_dword(bus, addr);
            let reg_write = i64::set_cp1_reg(cpu, reg, value as i64);
            cpu.wb.reg = reg_write.reg;
            cpu.wb.value = reg_write.value;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:016X}]", addr, value);
        }
        DcState::CacheOperation { op, vaddr } => {
            // TODO: TLB
            assert!((vaddr >> 30) == 2);
            let paddr = vaddr & 0x1fff_ffff;

            match op {
                0b00001 => {
                    cpu.dcache.index_write_back_invalidate(paddr, |line| {
                        bus.write_block(paddr & 0x1fff_fff0, line.data());
                        trace!("DCache Line at {:08X} written back to memory", paddr);
                    });
                }
                0b01000 => {
                    let tag = &cpu.cp0.tag_lo();
                    let ptag = tag.ptag_lo();
                    let valid = (tag.pstate() & 0b10) != 0;
                    cpu.icache.index_store_tag(paddr, ptag, valid);
                }
                0b01001 => {
                    let tag = &cpu.cp0.tag_lo();
                    let ptag = tag.ptag_lo();
                    let valid = (tag.pstate() & 0b10) != 0;
                    let dirty = (tag.pstate() & 0b01) != 0;
                    cpu.dcache.index_store_tag(paddr, ptag, valid, dirty);
                }
                0b01101 => {
                    cpu.dcache.create_dirty_exclusive(paddr, |line| {
                        bus.write_block(paddr & 0x1fff_fff0, line.data());
                        trace!("DCache Line at {:08X} written back to memory", paddr);
                    });
                }
                0b10000 => {
                    if let Some(line) = cpu.icache.find_mut(paddr) {
                        line.clear_valid_flag();
                        trace!("ICache Line at {:08X} invalidated", paddr);
                    }
                }
                0b10001 => {
                    if let Some(line) = cpu.dcache.find_mut(paddr) {
                        line.clear_valid_flag();
                        trace!("DCache Line at {:08X} invalidated", paddr);
                    }
                }
                0b10101 => {
                    cpu.dcache.hit_write_back_invalidate(paddr, |line| {
                        bus.write_block(paddr & 0x1fff_fff0, line.data());
                        trace!("DCache Line at {:08X} written back to memory", paddr);
                    });
                }
                0b11001 => {
                    if let Some(line) = cpu.dcache.find_mut(paddr) {
                        if line.is_dirty() {
                            bus.write_block(paddr & 0x1fff_fff0, line.data());
                            line.clear_dirty_flag();
                            trace!("DCache Line at {:08X} written back to memory", paddr);
                        }
                    }
                }
                op => todo!("Cache Operation: {:05b}", op),
            }
        }
        DcState::Nop => {
            cpu.wb.reg = 0;
            cpu.wb.op = None;
        }
    }
}
