use super::Cpu;
use tracing::trace;

pub fn bc1f<const LIKELY: bool>(cpu: &mut Cpu) {
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BC1F{} {}",
        cpu.pc[0],
        if LIKELY { "L" } else { "" },
        offset
    );

    cpu.stall += 1;

    cpu.branch::<LIKELY>(!cpu.cp1.status.c(), offset);
}

pub fn bc1t<const LIKELY: bool>(cpu: &mut Cpu) {
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BC1T{} {}",
        cpu.pc[0],
        if LIKELY { "L" } else { "" },
        offset
    );

    cpu.stall += 1;

    cpu.branch::<LIKELY>(cpu.cp1.status.c(), offset);
}
