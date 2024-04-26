use super::{Bus, Core};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    fn apply(bus: &mut impl Bus, value: i32, addr: u32);
}

pub struct Sb;
pub struct Sh;
pub struct Sw;

impl StoreOperator for Sb {
    const NAME: &'static str = "SB";

    fn apply(bus: &mut impl Bus, value: i32, addr: u32) {
        let value = value as u8;
        trace!("  [{:08X} <= {:02X}]", addr, value);
        bus.write_data(addr, value);
    }
}

impl StoreOperator for Sh {
    const NAME: &'static str = "SH";

    fn apply(bus: &mut impl Bus, value: i32, addr: u32) {
        let value = value as u16;
        trace!("  [{:08X} <= {:04X}]", addr, value);
        bus.write_data(addr, value);
    }
}

impl StoreOperator for Sw {
    const NAME: &'static str = "SW";

    fn apply(bus: &mut impl Bus, value: i32, addr: u32) {
        let value = value as u32;
        trace!("  [{:08X} <= {:08X}]", addr, value);
        bus.write_data(addr, value);
    }
}

pub fn store<Op: StoreOperator>(core: &Core, bus: &mut impl Bus) {
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
    let value = core.regs[rt];

    Op::apply(bus, value, address);
}
