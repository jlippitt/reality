use super::cp0::Cp0;
use super::{Cpu, DcState};

mod arithmetic;
mod bitwise;
mod control;
mod load;
mod store;

pub fn execute(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match word >> 26 {
        0o00 => special(cpu, pc, word),
        0o04 => control::beq::<false>(cpu, pc, word),
        0o05 => control::bne::<false>(cpu, pc, word),
        0o11 => arithmetic::addiu(cpu, pc, word),
        0o14 => bitwise::i_type::<bitwise::And>(cpu, pc, word),
        0o15 => bitwise::i_type::<bitwise::Or>(cpu, pc, word),
        0o16 => bitwise::i_type::<bitwise::Xor>(cpu, pc, word),
        0o17 => load::lui(cpu, pc, word),
        0o20 => Cp0::cop0(cpu, pc, word),
        0o24 => control::beq::<true>(cpu, pc, word),
        0o25 => control::bne::<true>(cpu, pc, word),
        0o43 => load::lw(cpu, pc, word),
        0o53 => store::sw(cpu, pc, word),
        opcode => todo!("CPU Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn special(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match word & 31 {
        0o10 => control::jr(cpu, pc, word),
        opcode => todo!("CPU Special Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}
