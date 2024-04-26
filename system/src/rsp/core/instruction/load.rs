use super::{Bus, Core};
use tracing::trace;

pub trait LoadOperator {
    const NAME: &'static str;
    fn apply(bus: &mut impl Bus, addr: u32) -> i32;
}

pub struct Lb;
pub struct Lbu;
pub struct Lh;
pub struct Lhu;
pub struct Lw;
pub struct Lwu;

impl LoadOperator for Lb {
    const NAME: &'static str = "LB";

    fn apply(bus: &mut impl Bus, addr: u32) -> i32 {
        let value = bus.read_data::<u8>(addr);
        trace!("  [{:08X} => {:02X}]", addr, value);
        value as i8 as i32
    }
}

impl LoadOperator for Lbu {
    const NAME: &'static str = "LBU";

    fn apply(bus: &mut impl Bus, addr: u32) -> i32 {
        let value = bus.read_data::<u8>(addr);
        trace!("  [{:08X} => {:02X}]", addr, value);
        value as i32
    }
}

impl LoadOperator for Lh {
    const NAME: &'static str = "LH";

    fn apply(bus: &mut impl Bus, addr: u32) -> i32 {
        let value = bus.read_data::<u16>(addr);
        trace!("  [{:08X} => {:04X}]", addr, value);
        value as i16 as i32
    }
}

impl LoadOperator for Lhu {
    const NAME: &'static str = "LHU";

    fn apply(bus: &mut impl Bus, addr: u32) -> i32 {
        let value = bus.read_data::<u16>(addr);
        trace!("  [{:08X} => {:04X}]", addr, value);
        value as i32
    }
}

impl LoadOperator for Lw {
    const NAME: &'static str = "LW";

    fn apply(bus: &mut impl Bus, addr: u32) -> i32 {
        let value = bus.read_data::<u32>(addr);
        trace!("  [{:08X} => {:08X}]", addr, value);
        value as i32
    }
}

impl LoadOperator for Lwu {
    const NAME: &'static str = "LWU";

    fn apply(bus: &mut impl Bus, addr: u32) -> i32 {
        let value = bus.read_data::<u32>(addr);
        trace!("  [{:08X} => {:08X}]", addr, value);
        value as i32
    }
}

pub fn lui(core: &mut Core) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let imm = (core.opcode[0] & 0xffff) as i16;

    trace!(
        "{:08X}: LUI {}, 0x{:04X}",
        core.pc[0],
        Core::REG_NAMES[rt],
        imm
    );

    core.set_reg(rt, (imm as i32) << 16);
}

pub fn load<Op: LoadOperator>(core: &mut Core, bus: &mut impl Bus) {
    let base = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let offset = (core.opcode[0] & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: {} {}, {}({})",
        core.pc[0],
        Op::NAME,
        Core::REG_NAMES[rt],
        offset,
        Core::REG_NAMES[base],
    );

    let address = core.regs[base].wrapping_add(offset) as u32;

    core.set_reg(rt, Op::apply(bus, address));
}
