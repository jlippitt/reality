use super::{Cpu, DcState};
use tracing::trace;

pub fn j<const LINK: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let offset = (word & 0x03ff_ffff) << 2;
    let target = (cpu.ex.pc.wrapping_add(4) & 0xf000_0000) | offset;

    trace!(
        "{:08X}: J{} 0x{:08X}",
        pc,
        if LINK { "AL" } else { "" },
        target
    );

    if cpu.delay == 0 {
        cpu.delay = 2;
        cpu.pc = target;
    }

    link::<LINK>(cpu)
}

pub fn jr(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;

    trace!("{:08X}: JR {}", pc, Cpu::REG_NAMES[rs]);

    if cpu.delay == 0 {
        cpu.delay = 2;
        cpu.pc = cpu.regs[rs] as u32;
    }

    DcState::Nop
}

pub fn jalr(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: JALR {}, {}",
        pc,
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
    );

    if cpu.delay == 0 {
        cpu.delay = 2;
        cpu.pc = cpu.regs[rs] as u32;
    }

    DcState::RegWrite {
        reg: rd,
        value: cpu.rf.pc.wrapping_add(4) as i64,
    }
}

pub fn beq<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BEQ{} {}, {}, {}",
        pc,
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] == cpu.regs[rt], offset);
    DcState::Nop
}

pub fn bne<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BNE{} {}, {}, {}",
        pc,
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] != cpu.regs[rt], offset);
    DcState::Nop
}

pub fn blez<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BLEZ{} {}, {}",
        pc,
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] <= 0, offset);
    DcState::Nop
}

pub fn bgtz<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BGTZ{} {}, {}",
        pc,
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] > 0, offset);
    DcState::Nop
}

pub fn bltz<const LINK: bool, const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BLTZ{}{} {}, {}",
        pc,
        if LINK { "AL" } else { "" },
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] < 0, offset);
    link::<LINK>(cpu)
}

pub fn bgez<const LINK: bool, const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BGEZ{}{} {}, {}",
        pc,
        if LINK { "AL" } else { "" },
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] >= 0, offset);
    link::<LINK>(cpu)
}

fn link<const LINK: bool>(cpu: &Cpu) -> DcState {
    if LINK {
        DcState::RegWrite {
            reg: 31,
            value: cpu.rf.pc.wrapping_add(4) as i64,
        }
    } else {
        DcState::Nop
    }
}
