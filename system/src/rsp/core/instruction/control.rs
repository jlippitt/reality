use super::Core;
use tracing::trace;

pub fn j<const LINK: bool>(core: &mut Core) {
    let offset = (core.opcode[0] & 0x03ff_ffff) << 2;

    trace!(
        "{:08X}: J{} 0x{:08X}",
        core.pc[0],
        if LINK { "AL" } else { "" },
        offset
    );

    if !core.delay[0] {
        core.delay[1] = true;
        core.pc[2] = offset & 0x0ffc;
    }

    if LINK {
        core.set_reg(31, (core.pc[1].wrapping_add(4) & 0x0ffc) as i32);
    }
}

pub fn jr(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;

    trace!("{:08X}: JR {}", core.pc[0], Core::REG_NAMES[rs]);

    if !core.delay[0] {
        core.delay[1] = true;
        core.pc[2] = (core.regs[rs] as u32) & 0x0ffc;
    }
}

pub fn jalr(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: JALR {}, {}",
        core.pc[0],
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
    );

    if !core.delay[0] {
        core.delay[1] = true;
        core.pc[2] = (core.regs[rs] as u32) & 0x0ffc;
    }

    core.set_reg(rd, (core.pc[1].wrapping_add(4) & 0x0ffc) as i32);
}

pub fn beq(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let offset = ((core.opcode[0] & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BEQ {}, {}, {}",
        core.pc[0],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
        offset
    );

    core.branch(core.regs[rs] == core.regs[rt], offset);
}

pub fn bne(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let offset = ((core.opcode[0] & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BNE {}, {}, {}",
        core.pc[0],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
        offset
    );

    core.branch(core.regs[rs] != core.regs[rt], offset);
}

pub fn blez(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let offset = ((core.opcode[0] & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BLEZ {}, {}",
        core.pc[0],
        Core::REG_NAMES[rs],
        offset
    );

    core.branch(core.regs[rs] <= 0, offset);
}

pub fn bgtz(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let offset = ((core.opcode[0] & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BGTZ {}, {}",
        core.pc[0],
        Core::REG_NAMES[rs],
        offset
    );

    core.branch(core.regs[rs] > 0, offset);
}

pub fn bltz<const LINK: bool>(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let offset = ((core.opcode[0] & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BLTZ{} {}, {}",
        core.pc[0],
        if LINK { "AL" } else { "" },
        Core::REG_NAMES[rs],
        offset
    );

    core.branch(core.regs[rs] < 0, offset);

    if LINK {
        core.set_reg(31, (core.pc[1].wrapping_add(4) & 0x0ffc) as i32);
    }
}

pub fn bgez<const LINK: bool>(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let offset = ((core.opcode[0] & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BGEZ{} {}, {}",
        core.pc[0],
        if LINK { "AL" } else { "" },
        Core::REG_NAMES[rs],
        offset
    );

    core.branch(core.regs[rs] >= 0, offset);

    if LINK {
        core.set_reg(31, (core.pc[1].wrapping_add(4) & 0x0ffc) as i32);
    }
}
