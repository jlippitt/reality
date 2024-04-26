use super::{Bus, Cp0, Cpu};
use tracing::trace;

pub trait LoadOperator {
    const NAME: &'static str;
    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) -> Option<i64>;
}

pub struct Lb;
pub struct Lbu;
pub struct Lh;
pub struct Lhu;
pub struct Lw;
pub struct Lwu;
pub struct Lwl;
pub struct Lwr;
pub struct Ld;
pub struct Ldl;
pub struct Ldr;
pub struct Ll;
pub struct Lld;

impl LoadOperator for Lb {
    const NAME: &'static str = "LB";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        let value = cpu.read_data::<u8>(bus, addr)?;
        trace!("  [{:08X} => {:02X}]", addr, value);
        Some(value as i8 as i64)
    }
}

impl LoadOperator for Lbu {
    const NAME: &'static str = "LBU";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        let value = cpu.read_data::<u8>(bus, addr)?;
        trace!("  [{:08X} => {:02X}]", addr, value);
        Some(value as i64)
    }
}

impl LoadOperator for Lh {
    const NAME: &'static str = "LH";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        assert!((addr & 1) == 0);
        let value = cpu.read_data::<u16>(bus, addr)?;
        trace!("  [{:08X} => {:04X}]", addr, value);
        Some(value as i16 as i64)
    }
}

impl LoadOperator for Lhu {
    const NAME: &'static str = "LHU";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        assert!((addr & 1) == 0);
        let value = cpu.read_data::<u16>(bus, addr)?;
        trace!("  [{:08X} => {:04X}]", addr, value);
        Some(value as i64)
    }
}

impl LoadOperator for Lw {
    const NAME: &'static str = "LW";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        assert!((addr & 3) == 0);
        let value = cpu.read_data::<u32>(bus, addr)?;
        trace!("  [{:08X} => {:08X}]", addr, value);
        Some(value as i32 as i64)
    }
}

impl LoadOperator for Lwu {
    const NAME: &'static str = "LWU";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        assert!((addr & 3) == 0);
        let value = cpu.read_data::<u32>(bus, addr)?;
        trace!("  [{:08X} => {:08X}]", addr, value);
        Some(value as i64)
    }
}

impl LoadOperator for Lwl {
    const NAME: &'static str = "LWL";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) -> Option<i64> {
        let value = cpu.read_data::<u32>(bus, addr & !3)?;
        trace!("  [{:08X} => {:08X}]", addr, value);
        let shift = (addr & 3) << 3;
        Some((cpu.regs[reg] as u32 & !(u32::MAX << shift) | (value << shift)) as i32 as i64)
    }
}

impl LoadOperator for Lwr {
    const NAME: &'static str = "LWR";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) -> Option<i64> {
        let value = cpu.read_data::<u32>(bus, addr & !3)?;
        trace!("  [{:08X} => {:08X}]", addr, value);
        let shift = (addr & 3 ^ 3) << 3;
        Some((cpu.regs[reg] as u32 & !(u32::MAX >> shift) | (value >> shift)) as i32 as i64)
    }
}

impl LoadOperator for Ld {
    const NAME: &'static str = "LD";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        assert!((addr & 7) == 0);
        let value = cpu.read_data::<u64>(bus, addr)?;
        trace!("  [{:08X} => {:016X}]", addr, value);
        Some(value as i64)
    }
}

impl LoadOperator for Ldl {
    const NAME: &'static str = "LDL";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) -> Option<i64> {
        let value = cpu.read_data::<u64>(bus, addr & !7)?;
        trace!("  [{:08X} => {:016X}]", addr, value);
        let shift = (addr & 7) << 3;
        Some((cpu.regs[reg] as u64 & !(u64::MAX << shift) | (value << shift)) as i64)
    }
}

impl LoadOperator for Ldr {
    const NAME: &'static str = "LDR";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, reg: usize, addr: u32) -> Option<i64> {
        // TODO: Stall cycles
        let value = cpu.read_data::<u64>(bus, addr & !7)?;
        trace!("  [{:08X} => {:016X}]", addr, value);
        let shift = (addr & 7 ^ 7) << 3;
        Some((cpu.regs[reg] as u64 & !(u64::MAX >> shift) | (value >> shift)) as i64)
    }
}

impl LoadOperator for Ll {
    const NAME: &'static str = "LL";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        assert!((addr & 3) == 0);
        let value = cpu.read_data::<u32>(bus, addr)?;
        trace!("  [{:08X} => {:08X}]", addr, value);
        // LLAddr is set to physical address
        // TODO: Remove this hack when TLB support is implemented
        cpu.cp0
            .write_reg(Cp0::LL_ADDR, ((addr & 0x1fff_ffff) >> 4) as i64);
        cpu.ll_bit = true;
        Some(value as i32 as i64)
    }
}

impl LoadOperator for Lld {
    const NAME: &'static str = "LLD";

    fn apply(cpu: &mut Cpu, bus: &mut impl Bus, _reg: usize, addr: u32) -> Option<i64> {
        assert!((addr & 7) == 0);
        let value = cpu.read_data::<u64>(bus, addr)?;
        trace!("  [{:08X} => {:016X}]", addr, value);
        // LLAddr is set to physical address
        // TODO: Remove this hack when TLB support is implemented
        cpu.cp0
            .write_reg(Cp0::LL_ADDR, ((addr & 0x1fff_ffff) >> 4) as i64);
        cpu.ll_bit = true;
        Some(value as i64)
    }
}

pub fn lui(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let imm = (cpu.opcode[0] & 0xffff) as i16;

    trace!(
        "{:08X}: LUI {}, 0x{:04X}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        imm
    );

    cpu.regs[rt] = ((imm as i32) << 16) as i64;
}

pub fn load<Op: LoadOperator>(cpu: &mut Cpu, bus: &mut impl Bus) {
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

    if let Some(value) = Op::apply(cpu, bus, rt, address) {
        cpu.regs[rt] = value;
    }
}
