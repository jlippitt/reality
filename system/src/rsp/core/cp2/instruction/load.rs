use super::{Bus, Core, Cp2, Vector};
use tracing::trace;

pub trait LoadOperator {
    const NAME: &'static str;
    const SHIFT: usize;
    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32);
}

pub struct Lbv;
pub struct Lsv;
pub struct Llv;
pub struct Ldv;
pub struct Lqv;
pub struct Lrv;
pub struct Lpv;
pub struct Luv;
pub struct Ltv;

impl LoadOperator for Lbv {
    const NAME: &'static str = "LBV";
    const SHIFT: usize = 0;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        let value = bus.read_data::<u8>(addr);
        trace!("  [{:08X} => {:02X}]", addr, value);
        let mut vector = cp2.reg(reg);
        vector.write(el, value);
        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Lsv {
    const NAME: &'static str = "LSV";
    const SHIFT: usize = 1;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        let value = bus.read_data::<u16>(addr);
        trace!("  [{:08X} => {:04X}]", addr, value);
        let mut vector = cp2.reg(reg);
        vector.write(el, value);
        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Llv {
    const NAME: &'static str = "LLV";
    const SHIFT: usize = 2;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        let value = bus.read_data::<u32>(addr);
        trace!("  [{:08X} => {:08X}]", addr, value);
        let mut vector = cp2.reg(reg);
        vector.write(el, value);
        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Ldv {
    const NAME: &'static str = "LDV";
    const SHIFT: usize = 3;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        let value = bus.read_data::<u64>(addr);
        trace!("  [{:08X} => {:016X}]", addr, value);
        let mut vector = cp2.reg(reg);
        vector.write(el, value);
        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Lqv {
    const NAME: &'static str = "LQV";
    const SHIFT: usize = 4;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        if el == 0 && (addr & 15) == 0 {
            // Aligned load
            let value = bus.read_data::<u128>(addr);
            trace!("  [{:08X} => {:032X}]", addr, value);
            cp2.set_reg(reg, value.into());
            return;
        }

        // Misaligned load
        let start = addr & !15;
        let value = bus.read_data::<u128>(start);
        trace!("  [{:08X} => {:032X}]", start, value);
        let bytes = value.to_be_bytes();
        let mut vector = cp2.reg(reg);

        for (index, byte) in bytes[(addr as usize & 15)..].iter().enumerate() {
            vector.write(el + index, *byte);
        }

        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Lrv {
    const NAME: &'static str = "LRV";
    const SHIFT: usize = 4;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, end: u32) {
        let addr = end & !15;
        let value = bus.read_data::<u128>(addr);
        trace!("  [{:08X} => {:032X}]", addr, value);
        let offset = end as usize & 15;
        let bytes = value.to_be_bytes();
        let mut vector = cp2.reg(reg);

        for (index, byte) in bytes[0..offset.min(16 - el)].iter().enumerate() {
            vector.write(el + (16 - offset) + index, *byte);
        }

        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Lpv {
    const NAME: &'static str = "LPV";
    const SHIFT: usize = 3;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        let start = addr & !7;
        let offset = (addr & 7).wrapping_sub(el as u32);
        let mut vector = Vector::default();

        for index in 0..8u32 {
            let byte_addr = start + (offset.wrapping_add(index) & 15);
            let value = bus.read_data::<u8>(byte_addr);
            trace!("  [{:08X} => {:02X}]", byte_addr, value);
            vector.set_lane(index as usize, (value as u16) << 8);
        }

        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Luv {
    const NAME: &'static str = "LUV";
    const SHIFT: usize = 3;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        let start = addr & !7;
        let offset = (addr & 7).wrapping_sub(el as u32);
        let mut vector = Vector::default();

        for index in 0..8 {
            let byte_addr = start + (offset.wrapping_add(index) & 15);
            let value = bus.read_data::<u8>(byte_addr);
            trace!("  [{:08X} => {:02X}]", byte_addr, value);
            vector.set_lane(index as usize, (value as u16) << 7);
        }

        cp2.set_reg(reg, vector);
    }
}

impl LoadOperator for Ltv {
    const NAME: &'static str = "LTV";
    const SHIFT: usize = 4;

    fn apply(cp2: &mut Cp2, bus: &impl Bus, reg: usize, el: usize, addr: u32) {
        let start = addr & !7;
        let end = start + 16;
        let mut byte_addr = start.wrapping_add((el as u32 + (addr & 8)) & 15);

        for index in 0..8 {
            let reg_index = (((el >> 1) + index) & 7) + (reg & !7);
            let mut vector = cp2.reg(reg_index);

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

            cp2.set_reg(reg_index, vector);
        }
    }
}

pub fn load<Op: LoadOperator>(core: &mut Core, bus: &impl Bus) {
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
        &mut core.cp2,
        bus,
        vt,
        el,
        core.regs[base].wrapping_add(offset) as u32,
    )
}

pub fn mtc2(core: &mut Core) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;
    let el = ((core.opcode[0] >> 7) & 15) as usize;

    trace!(
        "{:08X}: MTC2 {}, V{:02}[E{}]",
        core.pc[0],
        Core::REG_NAMES[rt],
        rd,
        el
    );

    let mut vector = core.cp2.reg(rd);
    vector.write(el, core.regs[rt] as u16);
    core.cp2.set_reg(rd, vector);
}

pub fn ctc2(core: &mut Core) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: CFC2 {}, {}",
        core.pc[0],
        Core::REG_NAMES[rt],
        Cp2::CONTROL_REG_NAMES[rd]
    );

    core.cp2.set_control_reg(rd, core.regs[rt]);
}
