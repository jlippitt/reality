use super::{Bus, Core, Cp2};
use tracing::trace;

pub trait StoreOperator {
    const NAME: &'static str;
    const SHIFT: usize;
    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32);
}

pub struct Sbv;
pub struct Ssv;
pub struct Slv;
pub struct Sdv;
pub struct Sqv;
pub struct Srv;
pub struct Spv;
pub struct Suv;
pub struct Stv;

impl StoreOperator for Sbv {
    const NAME: &'static str = "SBV";
    const SHIFT: usize = 0;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let value: u8 = cp2.reg(reg).read(el);
        trace!("  [{:08X} <= {:04X}]", addr, value);
        bus.write_data(addr, value);
    }
}

impl StoreOperator for Ssv {
    const NAME: &'static str = "SSV";
    const SHIFT: usize = 1;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let value: u16 = cp2.reg(reg).read(el);
        trace!("  [{:08X} <= {:04X}]", addr, value);
        bus.write_data(addr, value);
    }
}

impl StoreOperator for Slv {
    const NAME: &'static str = "SLV";
    const SHIFT: usize = 2;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let value: u32 = cp2.reg(reg).read(el);
        trace!("  [{:08X} <= {:08X}]", addr, value);
        bus.write_data(addr, value);
    }
}

impl StoreOperator for Sdv {
    const NAME: &'static str = "SDV";
    const SHIFT: usize = 3;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let value: u64 = cp2.reg(reg).read(el);
        trace!("  [{:08X} <= {:016X}]", addr, value);
        bus.write_data(addr, value);
    }
}

impl StoreOperator for Sqv {
    const NAME: &'static str = "SQV";
    const SHIFT: usize = 4;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let vector = cp2.reg(reg);

        if el == 0 && (addr & 15) == 0 {
            // Aligned store
            let value: u128 = vector.into();
            trace!("  [{:08X} <= {:032X}]", addr, value);
            bus.write_data(addr, value);
            return;
        }

        // Misaligned store
        let offset = addr as usize & 15;

        for index in 0..(16 - offset) {
            let byte: u8 = vector.read(el + index);
            trace!("  [{:08X} <= {:02X}]", addr + index as u32, byte);
            bus.write_data(addr + index as u32, byte);
        }
    }
}

impl StoreOperator for Srv {
    const NAME: &'static str = "SRV";
    const SHIFT: usize = 4;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, end: u32) {
        let vector = cp2.reg(reg);

        let addr = end & !15;
        let offset = end as usize & 15;

        for index in 0..offset {
            let byte: u8 = vector.read(el + (16 - offset) + index);
            trace!("  [{:08X} <= {:02X}]", addr + index as u32, byte);
            bus.write_data(addr + index as u32, byte);
        }
    }
}

impl StoreOperator for Spv {
    const NAME: &'static str = "SPV";
    const SHIFT: usize = 3;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let vector = cp2.reg(reg);

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
}

impl StoreOperator for Suv {
    const NAME: &'static str = "SUV";
    const SHIFT: usize = 3;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let vector = cp2.reg(reg);

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
}

impl StoreOperator for Stv {
    const NAME: &'static str = "STV";
    const SHIFT: usize = 4;

    fn apply(cp2: &Cp2, bus: &mut impl Bus, reg: usize, el: usize, addr: u32) {
        let start_addr = addr & !7;
        let start_el = 16 - (el & !1);
        let byte_offset = (addr & 7).wrapping_sub(el as u32 & !1);
        let mut index = 0;

        while index < 16 {
            let vector = cp2.reg((reg & !7) + (index >> 1));

            let low_addr = start_addr.wrapping_add(byte_offset.wrapping_add(index as u32) & 15);
            let low_byte: u8 = vector.read((start_el + index) & 15);
            trace!("  [{:08X} <= {:02X}]", low_addr, low_byte);
            bus.write_data(low_addr, low_byte);
            index += 1;

            let high_addr = start_addr.wrapping_add(byte_offset.wrapping_add(index as u32) & 15);
            let high_byte: u8 = vector.read((start_el + index) & 15);
            trace!("  [{:08X} <= {:02X}]", high_addr, high_byte);
            bus.write_data(high_addr, high_byte);
            index += 1;
        }
    }
}

pub fn store<Op: StoreOperator>(core: &Core, bus: &mut impl Bus) {
    let base = ((core.opcode[0] >> 21) & 31) as usize;
    let vt = ((core.opcode[0] >> 16) & 31) as usize;
    let el = ((core.opcode[0] >> 7) & 15) as usize;
    let offset =
        ((core.opcode[0] & 0x7f).wrapping_sub((core.opcode[0] & 0x40) << 1) as i32) << Op::SHIFT;

    trace!(
        "{:08X}: {} V{:02}[E{}], {}({})",
        core.pc[0],
        Op::NAME,
        vt,
        el,
        offset,
        Core::REG_NAMES[base],
    );

    Op::apply(
        &core.cp2,
        bus,
        vt,
        el,
        core.regs[base].wrapping_add(offset) as u32,
    )
}

pub fn mfc2(core: &mut Core) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;
    let el = ((core.opcode[0] >> 7) & 15) as usize;

    trace!(
        "{:08X}: MFC2 {}, V{:02}[E{}]",
        core.pc[0],
        Core::REG_NAMES[rt],
        rd,
        el
    );

    core.set_reg(rt, core.cp2.reg(rd).read::<u16>(el) as i16 as i32);
}

pub fn cfc2(core: &mut Core) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: CFC2 {}, {}",
        core.pc[0],
        Core::REG_NAMES[rt],
        Cp2::CONTROL_REG_NAMES[rd]
    );

    core.set_reg(rt, core.cp2.control_reg(rd));
}
