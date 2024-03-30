use super::{Cpu, DcState};

pub fn lui(cpu: &mut Cpu, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i64;

    println!(
        "{:08X}: LUI {}, 0x{:04X}",
        cpu.pc_debug,
        Cpu::REG_NAMES[rt],
        imm
    );

    DcState::RegWrite {
        reg: rt,
        value: imm << 16,
    }
}

pub fn lw(cpu: &mut Cpu, word: u32) -> DcState {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i64;

    println!(
        "{:08X}: LW {}, {}({})",
        cpu.pc_debug,
        Cpu::REG_NAMES[rt],
        offset,
        Cpu::REG_NAMES[base],
    );

    DcState::LoadWord {
        reg: rt,
        addr: cpu.regs[base].wrapping_add(offset) as u32,
    }
}
