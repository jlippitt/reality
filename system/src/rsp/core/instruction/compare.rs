use super::Core;
use tracing::trace;

pub fn slti(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let imm = (core.opcode[0] & 0xffff) as i16 as i32;

    trace!(
        "{:08X}: SLTI {}, {}, {}",
        core.pc[0],
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    core.set_reg(rt, (core.regs[rs] < imm) as i32);
}

pub fn sltiu(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let imm = (core.opcode[0] & 0xffff) as i16 as u32;

    trace!(
        "{:08X}: SLTIU {}, {}, {}",
        core.pc[0],
        Core::REG_NAMES[rt],
        Core::REG_NAMES[rs],
        imm
    );

    core.set_reg(rt, ((core.regs[rs] as u32) < imm) as i32);
}

pub fn slt(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLT {}, {}, {}",
        core.pc[0],
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    core.set_reg(rd, (core.regs[rs] < core.regs[rt]) as i32);
}

pub fn sltu(core: &mut Core) {
    let rs = ((core.opcode[0] >> 21) & 31) as usize;
    let rt = ((core.opcode[0] >> 16) & 31) as usize;
    let rd = ((core.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLTU {}, {}, {}",
        core.pc[0],
        Core::REG_NAMES[rd],
        Core::REG_NAMES[rs],
        Core::REG_NAMES[rt],
    );

    core.set_reg(rd, ((core.regs[rs] as u32) < (core.regs[rt] as u32)) as i32);
}
