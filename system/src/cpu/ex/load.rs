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
