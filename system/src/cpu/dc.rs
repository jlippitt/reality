use super::cp0::Cp0Register;
use super::{Bus, Cpu, WbOperation};
use tracing::trace;

#[derive(Debug)]
pub enum DcState {
    RegWrite { reg: usize, value: i64 },
    Cp0Write { reg: Cp0Register, value: i64 },
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
    StoreByte { value: u8, addr: u32 },
    StoreHalfword { value: u16, addr: u32 },
    StoreWord { value: u32, addr: u32 },
    StoreDoubleword { value: u64, addr: u32 },
    Nop,
}

pub fn execute(cpu: &mut Cpu, bus: &mut impl Bus) {
    match cpu.dc {
        DcState::RegWrite { reg, value } => {
            cpu.wb.reg = reg;
            cpu.wb.value = value;
            cpu.wb.op = None;
        }
        DcState::Cp0Write { reg, value } => {
            cpu.wb.reg = 0;
            // cpu.wb.value doesn't matter
            cpu.wb.op = Some(WbOperation::Cp0Write { reg, value });
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
        DcState::StoreDoubleword { value, addr } => {
            // TODO: Stall cycles
            assert!((addr & 3) == 0);
            cpu.wb.reg = 0;
            cpu.wb.op = None;
            trace!("  [{:08X} <= {:016X}]", addr, value);
            cpu.write_dword(bus, addr, value);
        }
        DcState::Nop => {
            cpu.wb.reg = 0;
            cpu.wb.op = None;
        }
    }
}
