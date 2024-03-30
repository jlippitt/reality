use super::{Cpu, DcState};

pub fn addiu(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i64;

    println!(
        "{:08X}: ADDIU {}, {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    DcState::RegWrite {
        reg: rt,
        value: cpu.regs[rs].wrapping_add(imm),
    }
}
