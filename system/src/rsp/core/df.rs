use super::cp2::Vector;
use super::{Bus, Core};
use tracing::trace;

#[derive(Debug)]
pub enum DfOperation {
    RegWrite {
        reg: usize,
        value: i32,
    },
    LoadByte {
        reg: usize,
        addr: u32,
    },
    LoadByteUnsigned {
        reg: usize,
        addr: u32,
    },
    LoadHalfword {
        reg: usize,
        addr: u32,
    },
    LoadHalfwordUnsigned {
        reg: usize,
        addr: u32,
    },
    LoadWord {
        reg: usize,
        addr: u32,
    },
    StoreByte {
        value: u8,
        addr: u32,
    },
    StoreHalfword {
        value: u16,
        addr: u32,
    },
    StoreWord {
        value: u32,
        addr: u32,
    },
    Cp0LoadReg {
        cp0_reg: usize,
        core_reg: usize,
    },
    Cp0StoreReg {
        cp0_reg: usize,
        value: i32,
    },
    Cp2LoadByte {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2LoadHalfword {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2LoadWord {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2LoadDoubleword {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2LoadQuadword {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2LoadQuadwordRight {
        reg: usize,
        el: usize,
        end: u32,
    },
    Cp2LoadPacked {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2LoadPackedUnsigned {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2LoadTranspose {
        reg: usize,
        el: usize,
        addr: u32,
    },
    Cp2StoreByte {
        value: u8,
        addr: u32,
    },
    Cp2StoreHalfword {
        value: u16,
        addr: u32,
    },
    Cp2StoreWord {
        value: u32,
        addr: u32,
    },
    Cp2StoreDoubleword {
        value: u64,
        addr: u32,
    },
    Cp2StoreQuadword {
        vector: Vector,
        el: usize,
        addr: u32,
    },
    Cp2StoreQuadwordRight {
        vector: Vector,
        el: usize,
        end: u32,
    },
    Cp2StorePacked {
        vector: Vector,
        el: usize,
        addr: u32,
    },
    Cp2StorePackedUnsigned {
        vector: Vector,
        el: usize,
        addr: u32,
    },
    Break,
    Nop,
}

pub fn execute(cpu: &mut Core, bus: &mut impl Bus) -> bool {
    match cpu.df {
        DfOperation::RegWrite { reg, value } => {
            cpu.wb.reg = reg;
            cpu.wb.value = value;
        }
        DfOperation::LoadByte { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i8 as i32;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DfOperation::LoadByteUnsigned { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DfOperation::LoadHalfword { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u16>(addr) as i16 as i64;
            cpu.wb.reg = reg;
            cpu.wb.value = value as i16 as i32;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DfOperation::LoadHalfwordUnsigned { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u16>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DfOperation::LoadWord { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u32>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DfOperation::StoreByte { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:02X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfOperation::StoreHalfword { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:04X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfOperation::StoreWord { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:08X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfOperation::Cp0LoadReg { cp0_reg, core_reg } => {
            cpu.wb.reg = core_reg;
            cpu.wb.value = bus.read_register(cp0_reg) as i32;
        }
        DfOperation::Cp0StoreReg { cp0_reg, value } => {
            cpu.wb.reg = 0;
            bus.write_register(cp0_reg, value as u32);
        }
        DfOperation::Cp2LoadByte { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:02X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfOperation::Cp2LoadHalfword { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u16>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:04X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfOperation::Cp2LoadWord { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u32>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:08X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfOperation::Cp2LoadDoubleword { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u64>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:016X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfOperation::Cp2LoadQuadword { reg, el, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;

            if el == 0 && (addr & 15) == 0 {
                // Aligned load
                let value = bus.read_data::<u128>(addr);
                trace!("  [{:08X} => {:032X}]", addr, value);
                cpu.cp2.set_reg(reg, value.into());
            } else {
                // Misaligned load
                let start = addr & !15;
                let value = bus.read_data::<u128>(start);
                trace!("  [{:08X} => {:032X}]", start, value);
                let bytes = value.to_be_bytes();
                let mut vector = cpu.cp2.reg(reg);

                for (index, byte) in bytes[(addr as usize & 15)..].iter().enumerate() {
                    vector.write(el + index, *byte);
                }

                cpu.cp2.set_reg(reg, vector);
            }
        }
        DfOperation::Cp2LoadQuadwordRight { reg, el, end } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;

            let addr = end & !15;
            let value = bus.read_data::<u128>(addr);
            trace!("  [{:08X} => {:032X}]", addr, value);
            let offset = end as usize & 15;
            let bytes = value.to_be_bytes();
            let mut vector = cpu.cp2.reg(reg);

            for (index, byte) in bytes[0..offset.min(16 - el)].iter().enumerate() {
                vector.write(el + (16 - offset) + index, *byte);
            }

            cpu.cp2.set_reg(reg, vector);
        }
        DfOperation::Cp2LoadPacked { reg, el, addr } => {
            cpu.wb.reg = 0;

            let start = addr & !7;
            let offset = (addr & 7).wrapping_sub(el as u32);
            let mut vector = Vector::default();

            for index in 0..8u32 {
                let byte_addr = start + (offset.wrapping_add(index) & 15);
                let value = bus.read_data::<u8>(byte_addr);
                trace!("  [{:08X} => {:02X}]", byte_addr, value);
                vector.set_lane(index as usize, (value as u16) << 8);
            }

            cpu.cp2.set_reg(reg, vector);
        }
        DfOperation::Cp2LoadPackedUnsigned { reg, el, addr } => {
            cpu.wb.reg = 0;

            let start = addr & !7;
            let offset = (addr & 7).wrapping_sub(el as u32);
            let mut vector = Vector::default();

            for index in 0..8 {
                let byte_addr = start + (offset.wrapping_add(index) & 15);
                let value = bus.read_data::<u8>(byte_addr);
                trace!("  [{:08X} => {:02X}]", byte_addr, value);
                vector.set_lane(index as usize, (value as u16) << 7);
            }

            cpu.cp2.set_reg(reg, vector);
        }
        DfOperation::Cp2LoadTranspose { reg, el, addr } => {
            cpu.wb.reg = 0;

            let start = addr & !7;
            let end = start + 16;
            let mut byte_addr = start.wrapping_add((el as u32 + (addr & 8)) & 15);

            for index in 0..8 {
                let reg_index = (((el >> 1) + index) & 7) + (reg & !7);
                let mut vector = cpu.cp2.reg(reg_index);

                let low_byte: u8 = bus.read_data(byte_addr);
                vector.write(index << 1, low_byte);
                byte_addr = byte_addr.wrapping_add(1);

                if byte_addr == end {
                    byte_addr = start;
                }

                let high_byte: u8 = bus.read_data(byte_addr);
                vector.write((index << 1) + 1, high_byte);
                byte_addr = byte_addr.wrapping_add(1);

                if byte_addr == end {
                    byte_addr = start;
                }

                cpu.cp2.set_reg(reg_index, vector);
            }
        }
        DfOperation::Cp2StoreByte { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:02X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfOperation::Cp2StoreHalfword { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:04X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfOperation::Cp2StoreWord { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:08X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfOperation::Cp2StoreDoubleword { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:016X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfOperation::Cp2StoreQuadword { vector, el, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;

            if el == 0 && (addr & 15) == 0 {
                // Aligned store
                let value: u128 = vector.into();
                trace!("  [{:08X} <= {:032X}]", addr, value);
                bus.write_data(addr, value);
            } else {
                // Misaligned store
                let offset = addr as usize & 15;

                for index in 0..(16 - offset) {
                    let byte: u8 = vector.read(el + index);
                    bus.write_data(addr + index as u32, byte);
                    trace!("  [{:08X} <= {:02X}]", addr + index as u32, byte);
                }
            }
        }
        DfOperation::Cp2StoreQuadwordRight { vector, el, end } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;

            let addr = end & !15;
            let offset = end as usize & 15;

            for index in 0..offset {
                let byte: u8 = vector.read(el + (16 - offset) + index);
                bus.write_data(addr + index as u32, byte);
                trace!("  [{:08X} <= {:02X}]", addr + index as u32, byte);
            }
        }
        DfOperation::Cp2StorePacked { vector, el, addr } => {
            cpu.wb.reg = 0;

            for index in 0..8u32 {
                let offset = (index + el as u32) & 0x0f;

                let value = if offset < 8 {
                    (vector.lane(offset as usize) >> 8) as u8
                } else {
                    (vector.lane(offset as usize & 7) >> 7) as u8
                };

                let byte_addr = addr.wrapping_add(index);
                bus.write_data::<u8>(byte_addr, value);
                trace!("  [{:08X} <= {:02X}]", byte_addr, value);
            }
        }
        DfOperation::Cp2StorePackedUnsigned { vector, el, addr } => {
            cpu.wb.reg = 0;

            for index in 0..8u32 {
                let offset = (index + el as u32) & 0x0f;

                let value = if offset < 8 {
                    (vector.lane(offset as usize) >> 7) as u8
                } else {
                    (vector.lane(offset as usize & 7) >> 8) as u8
                };

                let byte_addr = addr.wrapping_add(index);
                bus.write_data::<u8>(byte_addr, value);
                trace!("  [{:08X} <= {:02X}]", byte_addr, value);
            }
        }
        DfOperation::Break => {
            bus.break_();
            cpu.wb.reg = 0;
            cpu.pc = cpu.ex.pc;
            cpu.rf.word = 0;
            cpu.ex.word = 0;
            cpu.df = DfOperation::Nop;
            return true;
        }
        DfOperation::Nop => {
            cpu.wb.reg = 0;
        }
    }

    false
}
