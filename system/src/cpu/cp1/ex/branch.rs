use super::{Cpu, DcOperation};
use tracing::trace;

pub fn bc1f<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BC1F{} {}",
        pc,
        if LIKELY { "L" } else { "" },
        offset
    );

    cpu.branch::<LIKELY>(!cpu.cp1.status.c(), offset);
    DcOperation::Nop
}

pub fn bc1t<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BC1T{} {}",
        pc,
        if LIKELY { "L" } else { "" },
        offset
    );

    cpu.branch::<LIKELY>(cpu.cp1.status.c(), offset);
    DcOperation::Nop
}
