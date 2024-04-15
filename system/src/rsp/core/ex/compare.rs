use super::{Core, DfState};
use tracing::trace;

pub fn slti(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: SLTI {}, {}, {}",
        pc,
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    DfState::RegWrite {
        reg: rt,
        value: (cpu.regs[rs] < imm) as i32,
    }
}

pub fn sltiu(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as u32;

    trace!(
        "{:08X}: SLTIU {}, {}, {}",
        pc,
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    DfState::RegWrite {
        reg: rt,
        value: ((cpu.regs[rs] as u32) < imm) as i32,
    }
}

pub fn slt(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLT {}, {}, {}",
        pc,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    DfState::RegWrite {
        reg: rd,
        value: (cpu.regs[rs] < cpu.regs[rt]) as i32,
    }
}

pub fn sltu(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLTU {}, {}, {}",
        pc,
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    DfState::RegWrite {
        reg: rd,
        value: ((cpu.regs[rs] as u32) < (cpu.regs[rt] as u32)) as i32,
    }
}
