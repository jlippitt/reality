use super::{Bus, Cpu};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32);
}

pub struct Sb;
pub struct Sh;
pub struct Sw;
pub struct Swl;
pub struct Swr;
pub struct Sd;
pub struct Sdl;
pub struct Sdr;
pub struct Sc;
pub struct Scd;

impl StoreOperator for Sb {
    const NAME: &'static str = "SB";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        let value = cpu.regs[reg] as u8;
        trace!("  [{:08X} <= {:02X}]", addr, value);
        cpu.write_data(bus, addr, value);
    }
}

impl StoreOperator for Sh {
    const NAME: &'static str = "SH";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        assert!((addr & 1) == 0);
        let value = cpu.regs[reg] as u16;
        trace!("  [{:08X} <= {:04X}]", addr, value);
        cpu.write_data(bus, addr, value);
    }
}

impl StoreOperator for Sw {
    const NAME: &'static str = "SW";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        assert!((addr & 3) == 0);
        let value = cpu.regs[reg] as u32;
        trace!("  [{:08X} <= {:08X}]", addr, value);
        cpu.write_data(bus, addr, value);
    }
}

impl StoreOperator for Swl {
    const NAME: &'static str = "SWL";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        let value = cpu.regs[reg] as u32;
        trace!("  [{:08X} <= {:08X}]", addr, value);

        match addr & 3 {
            0 => cpu.write_data(bus, addr & !3, value),
            1 => {
                cpu.write_data(bus, addr & !3 | 1, (value >> 24) as u8);
                cpu.write_data(bus, addr & !3 | 2, (value >> 8) as u16);
            }
            2 => cpu.write_data(bus, addr & !3 | 2, (value >> 16) as u16),
            _ => cpu.write_data(bus, addr & !3 | 3, (value >> 24) as u8),
        }
    }
}

impl StoreOperator for Swr {
    const NAME: &'static str = "SWR";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        let value = cpu.regs[reg] as u32;
        trace!("  [{:08X} <= {:08X}]", addr, value);

        match addr & 3 {
            0 => cpu.write_data(bus, addr & !3, value as u8),
            1 => cpu.write_data(bus, addr & !3, value as u16),
            2 => {
                cpu.write_data(bus, addr & !3, (value >> 8) as u16);
                cpu.write_data(bus, addr & !3 | 2, value as u8);
            }
            _ => cpu.write_data(bus, addr & !3, value),
        }
    }
}

impl StoreOperator for Sd {
    const NAME: &'static str = "SD";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        assert!((addr & 7) == 0);
        let value = cpu.regs[reg] as u64;
        trace!("  [{:08X} <= {:016X}]", addr, value);
        cpu.write_data(bus, addr, value);
    }
}

impl StoreOperator for Sdl {
    const NAME: &'static str = "SDL";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        let value = cpu.regs[reg] as u64;
        trace!("  [{:08X} <= {:08X}]", addr, value);

        match addr & 7 {
            0 => cpu.write_data(bus, addr & !7, value),
            1 => {
                cpu.write_data(bus, addr & !7 | 1, (value >> 56) as u8);
                cpu.write_data(bus, addr & !7 | 2, (value >> 40) as u16);
                cpu.write_data(bus, addr & !7 | 4, (value >> 8) as u32);
            }
            2 => {
                cpu.write_data(bus, addr & !7 | 2, (value >> 48) as u16);
                cpu.write_data(bus, addr & !7 | 4, (value >> 16) as u32);
            }
            3 => {
                cpu.write_data(bus, addr & !7 | 3, (value >> 56) as u8);
                cpu.write_data(bus, addr & !7 | 4, (value >> 24) as u32);
            }
            4 => cpu.write_data(bus, addr & !7 | 4, (value >> 32) as u32),
            5 => {
                cpu.write_data(bus, addr & !7 | 5, (value >> 56) as u8);
                cpu.write_data(bus, addr & !7 | 6, (value >> 40) as u16);
            }
            6 => cpu.write_data(bus, addr & !7 | 6, (value >> 48) as u16),
            _ => cpu.write_data(bus, addr & !7 | 7, (value >> 56) as u8),
        }
    }
}

impl StoreOperator for Sdr {
    const NAME: &'static str = "SDR";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        let value = cpu.regs[reg] as u64;
        trace!("  [{:08X} <= {:08X}]", addr, value);

        match addr & 7 {
            0 => cpu.write_data(bus, addr & !7, value as u8),
            1 => cpu.write_data(bus, addr & !7, value as u16),
            2 => {
                cpu.write_data(bus, addr & !7, (value >> 8) as u16);
                cpu.write_data(bus, addr & !7 | 2, value as u8);
            }
            3 => cpu.write_data(bus, addr & !7, value as u32),
            4 => {
                cpu.write_data(bus, addr & !7, (value >> 8) as u32);
                cpu.write_data(bus, addr & !7 | 4, value as u8);
            }
            5 => {
                cpu.write_data(bus, addr & !7, (value >> 16) as u32);
                cpu.write_data(bus, addr & !7 | 4, value as u16);
            }
            6 => {
                cpu.write_data(bus, addr & !7, (value >> 24) as u32);
                cpu.write_data(bus, addr & !7 | 4, (value >> 8) as u16);
                cpu.write_data(bus, addr & !7 | 6, value as u8);
            }
            _ => cpu.write_data(bus, addr & !7, value),
        }
    }
}

impl StoreOperator for Sc {
    const NAME: &'static str = "SC";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        assert!((addr & 3) == 0);
        cpu.regs[reg] = cpu.ll_bit as i64;

        if cpu.ll_bit {
            let value = cpu.regs[reg] as u32;
            trace!("  [{:08X} <= {:08X}]", addr, value);
            cpu.write_data(bus, addr, value);
        }
    }
}

impl StoreOperator for Scd {
    const NAME: &'static str = "SCD";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) {
        assert!((addr & 7) == 0);
        cpu.regs[reg] = cpu.ll_bit as i64;

        if cpu.ll_bit {
            let value = cpu.regs[reg] as u64;
            trace!("  [{:08X} <= {:016X}]", addr, value);
            cpu.write_data(bus, addr, value);
        }
    }
}

pub fn store<Op: StoreOperator>(cpu: &mut Cpu, bus: &mut impl Bus) {
    let base = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let offset = (cpu.opcode[0] & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: {} {}, {}({})",
        cpu.pc[0],
        Op::NAME,
        Cpu::REG_NAMES[rt],
        offset,
        Cpu::REG_NAMES[base],
    );

    let address = cpu.regs[base].wrapping_add(offset) as u32;

    Op::apply(cpu, bus, rt, address);
}
