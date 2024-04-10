use super::{Core, DfState, REG_NAMES};
use tracing::trace;

pub fn cop0(core: &mut Core, pc: u32, word: u32) -> DfState {
    match (word >> 21) & 31 {
        0o00 => mfc0(core, pc, word),
        0o04 => mtc0(core, pc, word),
        opcode => todo!("RSP COP0 Opcode '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn mfc0(_core: &mut Core, pc: u32, word: u32) -> DfState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: MFC0 {}, {}",
        pc,
        Core::REG_NAMES[rt],
        REG_NAMES[rd]
    );

    DfState::Cp0LoadReg {
        cp0_reg: rd,
        core_reg: rt,
    }
}

pub fn mtc0(core: &mut Core, pc: u32, word: u32) -> DfState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: MTC0 {}, {}",
        pc,
        Core::REG_NAMES[rt],
        REG_NAMES[rd]
    );

    DfState::Cp0StoreReg {
        cp0_reg: rd,
        value: core.regs[rt],
    }
}
