use super::cp2::Vector;
use super::{Bus, Core};
use tracing::trace;

#[derive(Debug)]
pub enum DfState {
    RegWrite { reg: usize, value: i32 },
    LoadByte { reg: usize, addr: u32 },
    LoadByteUnsigned { reg: usize, addr: u32 },
    LoadHalfword { reg: usize, addr: u32 },
    LoadHalfwordUnsigned { reg: usize, addr: u32 },
    LoadWord { reg: usize, addr: u32 },
    StoreByte { value: u8, addr: u32 },
    StoreHalfword { value: u16, addr: u32 },
    StoreWord { value: u32, addr: u32 },
    Cp0LoadReg { cp0_reg: usize, core_reg: usize },
    Cp0StoreReg { cp0_reg: usize, value: i32 },
    Cp2LoadByte { reg: usize, el: usize, addr: u32 },
    Cp2LoadHalfword { reg: usize, el: usize, addr: u32 },
    Cp2LoadWord { reg: usize, el: usize, addr: u32 },
    Cp2LoadDoubleword { reg: usize, el: usize, addr: u32 },
    Cp2LoadQuadword { reg: usize, el: usize, addr: u32 },
    Cp2StoreByte { value: u8, addr: u32 },
    Cp2StoreHalfword { value: u16, addr: u32 },
    Cp2StoreWord { value: u32, addr: u32 },
    Cp2StoreDoubleword { value: u64, addr: u32 },
    Cp2StoreQuadword { vec: Vector, el: usize, addr: u32 },
    Break,
    Nop,
}

pub fn execute(cpu: &mut Core, bus: &mut impl Bus) -> bool {
    match cpu.df {
        DfState::RegWrite { reg, value } => {
            cpu.wb.reg = reg;
            cpu.wb.value = value;
        }
        DfState::LoadByte { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i8 as i32;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DfState::LoadByteUnsigned { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DfState::LoadHalfword { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u16>(addr) as i16 as i64;
            cpu.wb.reg = reg;
            cpu.wb.value = value as i16 as i32;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DfState::LoadHalfwordUnsigned { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u16>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DfState::LoadWord { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u32>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DfState::StoreByte { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:02X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::StoreHalfword { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:04X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::StoreWord { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:08X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::Cp0LoadReg { cp0_reg, core_reg } => {
            cpu.wb.reg = core_reg;
            cpu.wb.value = bus.read_register(cp0_reg) as i32;
        }
        DfState::Cp0StoreReg { cp0_reg, value } => {
            cpu.wb.reg = 0;
            bus.write_register(cp0_reg, value as u32);
        }
        DfState::Cp2LoadByte { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:02X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfState::Cp2LoadHalfword { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u16>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:04X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfState::Cp2LoadWord { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u32>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:08X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfState::Cp2LoadDoubleword { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u64>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:016X}]", addr, value);
            let mut vector = cpu.cp2.reg(reg);
            vector.write(el, value);
            cpu.cp2.set_reg(reg, vector);
        }
        DfState::Cp2LoadQuadword { reg, el, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u128>(addr);
            cpu.wb.reg = 0;
            trace!("  [{:08X} => {:032X}]", addr, value);

            if el == 0 && (addr & 7) == 0 {
                // Aligned load
                cpu.cp2.set_reg(reg, value.into());
            } else {
                todo!("Misaligned quadword load");
            }
        }
        DfState::Cp2StoreByte { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:02X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::Cp2StoreHalfword { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:04X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::Cp2StoreWord { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:08X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::Cp2StoreDoubleword { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            trace!("  [{:08X} <= {:016X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::Cp2StoreQuadword { vec, el, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;

            if el == 0 && (addr & 7) == 0 {
                // Aligned store
                let value: u128 = vec.into();
                trace!("  [{:08X} <= {:032X}]", addr, value);
                bus.write_data(addr, value);
            } else {
                todo!("Misaligned quadword load");
            }
        }
        DfState::Break => {
            bus.break_();
            cpu.wb.reg = 0;
            cpu.rf.word = 0;
            cpu.ex.word = 0;
            cpu.df = DfState::Nop;
            return true;
        }
        DfState::Nop => {
            cpu.wb.reg = 0;
        }
    }

    false
}
