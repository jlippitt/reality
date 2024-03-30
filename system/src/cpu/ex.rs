use super::{Cpu, DcState};

pub fn execute(cpu: &mut Cpu, word: u32) {
    match word >> 26 {
        0o17 => lui(cpu, word),
        opcode => todo!("Opcode {:02o}", opcode),
    }
}

fn lui(cpu: &mut Cpu, word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i64;
    println!("LUI {}, 0x{:04X}", Cpu::REG_NAMES[rt], imm);
    cpu.dc = DcState::RegWrite {
        reg: rt,
        value: imm << 16,
    };
}
