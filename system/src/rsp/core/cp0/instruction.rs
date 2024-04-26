use super::{Bus, Core, REG_NAMES};
use tracing::trace;

pub fn cop0(core: &mut Core, bus: &mut impl Bus) {
    match (core.opcode[0] >> 21) & 31 {
        0o00 => mfc0(core, bus),
        0o04 => mtc0(core, bus),
        opcode => todo!("RSP COP0 Opcode '{:02o}' at {:08X}", opcode, core.pc[0]),
    }
}

pub fn mfc0(core: &mut Core, bus: &mut impl Bus) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: MFC0 {}, {}",
        core.pc[0],
        Core::REG_NAMES[rt],
        REG_NAMES[rd]
    );

    let value = bus.read_register(rd) as i32;
    core.set_reg(rt, value);
}

pub fn mtc0(core: &mut Core, bus: &mut impl Bus) {
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: MTC0 {}, {}",
        core.pc[0],
        Core::REG_NAMES[rt],
        REG_NAMES[rd]
    );

    bus.write_register(rd, core.regs[rt] as u32);
}
