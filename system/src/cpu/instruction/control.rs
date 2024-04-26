use super::Cpu;
use tracing::trace;

pub fn j<const LINK: bool>(cpu: &mut Cpu) {
    let offset = (cpu.opcode[0] & 0x03ff_ffff) << 2;
    let target = (cpu.pc[0].wrapping_add(4) & 0xf000_0000) | offset;

    trace!(
        "{:08X}: J{} 0x{:08X}",
        cpu.pc[0],
        if LINK { "AL" } else { "" },
        target
    );

    if !cpu.delay[0] {
        cpu.delay[1] = true;
        cpu.pc[2] = target;
    }

    if LINK {
        cpu.regs[31] = cpu.pc[1].wrapping_add(4) as i32 as i64;
    }
}

pub fn jr(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;

    trace!("{:08X}: JR {}", cpu.pc[0], Cpu::REG_NAMES[rs]);

    if !cpu.delay[0] {
        cpu.delay[1] = true;
        cpu.pc[2] = cpu.regs[rs] as u32;
    }
}

pub fn jalr(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: JALR {}, {}",
        cpu.pc[0],
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
    );

    if !cpu.delay[0] {
        cpu.delay[1] = true;
        cpu.pc[2] = cpu.regs[rs] as u32;
    }

    cpu.regs[rd] = cpu.pc[1].wrapping_add(4) as i32 as i64;
}

pub fn beq<const LIKELY: bool>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BEQ{} {}, {}, {}",
        cpu.pc[0],
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] == cpu.regs[rt], offset);

    if rs == 0 && rt == 0 && offset == -4 {
        cpu.busy_wait = true;
    }
}

pub fn bne<const LIKELY: bool>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BNE{} {}, {}, {}",
        cpu.pc[0],
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] != cpu.regs[rt], offset);
}

pub fn blez<const LIKELY: bool>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BLEZ{} {}, {}",
        cpu.pc[0],
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] <= 0, offset);

    if rs == 0 && offset == -4 {
        cpu.busy_wait = true;
    }
}

pub fn bgtz<const LIKELY: bool>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BGTZ{} {}, {}",
        cpu.pc[0],
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] > 0, offset);
}

pub fn bltz<const LINK: bool, const LIKELY: bool>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BLTZ{}{} {}, {}",
        cpu.pc[0],
        if LINK { "AL" } else { "" },
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] < 0, offset);

    if LINK {
        cpu.regs[31] = cpu.pc[1].wrapping_add(4) as i32 as i64;
    }
}

pub fn bgez<const LINK: bool, const LIKELY: bool>(cpu: &mut Cpu) {
    let rs = ((cpu.opcode[0] >> 21) & 31) as usize;
    let offset = ((cpu.opcode[0] & 0xffff) as i16 as i64) << 2;

    trace!(
        "{:08X}: BGEZ{}{} {}, {}",
        cpu.pc[0],
        if LINK { "AL" } else { "" },
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        offset
    );

    cpu.branch::<LIKELY>(cpu.regs[rs] >= 0, offset);

    if LINK {
        cpu.regs[31] = cpu.pc[1].wrapping_add(4) as i32 as i64;
    } else if rs == 0 && offset == -4 {
        cpu.busy_wait = true;
    }
}
