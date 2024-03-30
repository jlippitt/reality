use super::cp0::Cp0;
use super::{Cpu, DcState};

mod bitwise;
mod load;

pub fn execute(cpu: &mut Cpu, word: u32) -> DcState {
    match word >> 26 {
        0o14 => bitwise::i_type::<bitwise::And>(cpu, word),
        0o15 => bitwise::i_type::<bitwise::Or>(cpu, word),
        0o16 => bitwise::i_type::<bitwise::Xor>(cpu, word),
        0o17 => load::lui(cpu, word),
        0o20 => Cp0::cop0(cpu, word),
        opcode => todo!("CPU Opcode: '{:02o}' at {:08X}", opcode, cpu.pc_debug),
    }
}
