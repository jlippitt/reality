use super::{Cpu, DcState};
use tracing::trace;

pub fn bc1f<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BC1F{} {}",
        pc,
        if LIKELY { "L" } else { "" },
        offset
    );

    cpu.branch::<LIKELY>(!cpu.cp1.status.c(), offset);
    DcState::Nop
}

pub fn bc1t<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BC1T{} {}",
        pc,
        if LIKELY { "L" } else { "" },
        offset
    );

    cpu.branch::<LIKELY>(cpu.cp1.status.c(), offset);
    DcState::Nop
}
