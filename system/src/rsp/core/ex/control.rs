use super::{Core, DfState};
use tracing::trace;

pub fn j<const LINK: bool>(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let offset = (word & 0x03ff_ffff) << 2;
    let target = offset & 0x0fff;

    trace!(
        "{:08X}: J{} 0x{:08X}",
        pc,
        if LINK { "AL" } else { "" },
        target
    );

    cpu.pc = target;
    link::<LINK>(cpu)
}

pub fn jr<const LINK: bool>(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;

    trace!(
        "{:08X}: J{}R {}",
        pc,
        if LINK { "AL" } else { "" },
        Core::REG_NAMES[rs]
    );

    cpu.pc = (cpu.regs[rs] as u32) & 0x0fff;
    link::<LINK>(cpu)
}

pub fn beq(cpu: &mut Core, pc: u32, word: u32) -> DfState {
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

    cpu.branch(cpu.regs[rs] == cpu.regs[rt], offset);
    DfState::Nop
}

pub fn bne(cpu: &mut Core, pc: u32, word: u32) -> DfState {
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

    cpu.branch(cpu.regs[rs] != cpu.regs[rt], offset);
    DfState::Nop
}

pub fn blez(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!("{:08X}: BLEZ {}, {}", pc, Core::REG_NAMES[rs], offset);

    cpu.branch(cpu.regs[rs] <= 0, offset);
    DfState::Nop
}

pub fn bgtz(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!("{:08X}: BGTZ {}, {}", pc, Core::REG_NAMES[rs], offset);

    cpu.branch(cpu.regs[rs] > 0, offset);
    DfState::Nop
}

pub fn bltz<const LINK: bool>(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BLTZ{} {}, {}",
        pc,
        if LINK { "AL" } else { "" },
        Core::REG_NAMES[rs],
        offset
    );

    cpu.branch(cpu.regs[rs] < 0, offset);
    link::<LINK>(cpu)
}

pub fn bgez<const LINK: bool>(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i32) << 2;

    trace!(
        "{:08X}: BGEZ{} {}, {}",
        pc,
        if LINK { "AL" } else { "" },
        Core::REG_NAMES[rs],
        offset
    );

    cpu.branch(cpu.regs[rs] >= 0, offset);
    link::<LINK>(cpu)
}

fn link<const LINK: bool>(cpu: &Core) -> DfState {
    if LINK {
        DfState::RegWrite {
            reg: 31,
            value: cpu.ex.pc.wrapping_add(8) as i32,
        }
    } else {
        DfState::Nop
    }
}
