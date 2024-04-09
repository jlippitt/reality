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
    //Cp0RegWrite { reg: usize, value: i32 },
    Break,
    Nop,
}

pub fn execute(cpu: &mut Core, bus: &mut impl Bus) {
    match cpu.dc {
        DfState::RegWrite { reg, value } => {
            cpu.wb.reg = reg;
            cpu.wb.value = value;
            cpu.wb.op = None;
        }
        DfState::LoadByte { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i8 as i32;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DfState::LoadByteUnsigned { reg, addr } => {
            // TODO: Stall cycles
            let value = bus.read_data::<u8>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:02X}]", addr, value);
        }
        DfState::LoadHalfword { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 1) == 0);
            let value = bus.read_data::<u16>(addr) as i16 as i64;
            cpu.wb.reg = reg;
            cpu.wb.value = value as i16 as i32;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DfState::LoadHalfwordUnsigned { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 1) == 0);
            let value = bus.read_data::<u16>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:04X}]", addr, value);
        }
        DfState::LoadWord { reg, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            let value = bus.read_data::<u32>(addr);
            cpu.wb.reg = reg;
            cpu.wb.value = value as i32;
            cpu.wb.op = None;
            trace!("  [{:08X} => {:08X}]", addr, value);
        }
        DfState::StoreByte { value, addr } => {
            // TODO: Stall cycles
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:02X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::StoreHalfword { value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 1) == 0);
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:04X}]", addr, value);
            bus.write_data(addr, value);
        }
        DfState::StoreWord { value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:08X}]", addr, value);
            bus.write_data(addr, value);
        }
        // DcState::Cp0RegWrite { reg, value } => {
        //     cpu.wb.reg = 0;
        //     cpu.wb.op = Some(WbOperation::Cp0RegWrite { reg, value });
        // }
        DfState::Break => {
            bus.break_();
        }
        DfState::Nop => {
            cpu.wb.reg = 0;
            cpu.wb.op = None;
        }
    }
}
