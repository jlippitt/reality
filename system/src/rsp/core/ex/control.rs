use super::{Core, DfState};
use tracing::trace;

pub fn j<const LINK: bool>(core: &mut Core, pc: u32, word: u32) -> DfState {
    let offset = (word & 0x03ff_ffff) << 2;
    let target = offset & 0x0fff;

    trace!(
        "{:08X}: J{} 0x{:08X}",
        pc,
        if LINK { "AL" } else { "" },
        target
    );

    if core.delay == 0 {
        core.delay = 2;
        core.pc = target & 0xfffc;
    }

    link::<LINK>(core)
}

pub fn jr(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;

    trace!("{:08X}: JR {}", pc, Core::REG_NAMES[rs]);

    if core.delay == 0 {
        core.delay = 2;
        core.pc = (core.regs[rs] as u32) & 0x0ffc;
    }

    DfState::Nop
}

pub fn jalr(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: JALR {}, {}",
        pc,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
    );

    if core.delay == 0 {
        core.delay = 2;
        core.pc = (core.regs[rs] as u32) & 0x0ffc;
    }

    DfState::RegWrite {
        reg: rd,
        value: ((core.rf.pc + 4) & 0x0ffc) as i32,
    }
}

pub fn beq(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BEQ {}, {}, {}",
        pc,
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
        offset
    );

    core.branch(core.regs[rs] == core.regs[rt], offset);
    DfState::Nop
}

pub fn bne(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BNE {}, {}, {}",
        pc,
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
        offset
    );

    core.branch(core.regs[rs] != core.regs[rt], offset);
    DfState::Nop
}

pub fn blez(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!("{:08X}: BLEZ {}, {}", pc, Core::REG_NAMES[rs], offset);

    core.branch(core.regs[rs] <= 0, offset);
    DfState::Nop
}

pub fn bgtz(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!("{:08X}: BGTZ {}, {}", pc, Core::REG_NAMES[rs], offset);

    core.branch(core.regs[rs] > 0, offset);
    DfState::Nop
}

pub fn bltz<const LINK: bool>(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BLTZ{} {}, {}",
        pc,
        if LINK { "AL" } else { "" },
        Core::REG_NAMES[rs],
        offset
    );

    core.branch(core.regs[rs] < 0, offset);
    link::<LINK>(core)
}

pub fn bgez<const LINK: bool>(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BGEZ{} {}, {}",
        pc,
        if LINK { "AL" } else { "" },
        Core::REG_NAMES[rs],
        offset
    );

    core.branch(core.regs[rs] >= 0, offset);
    link::<LINK>(core)
}

fn link<const LINK: bool>(core: &Core) -> DfState {
    if LINK {
        DfState::RegWrite {
            reg: 31,
            value: ((core.rf.pc + 4) & 0x0fff) as i32,
        }
    } else {
        DfState::Nop
    }
}
