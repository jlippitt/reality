pub use misc::cache;

use super::Cp0;
use super::{Cpu, DcState};

mod misc;
mod tlb;
mod transfer;

pub fn cop0(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match (word >> 21) & 31 {
        0o00 => transfer::mfc0(cpu, pc, word),
        0o04 => transfer::mtc0(cpu, pc, word),
        0o20..=0o37 => match word & 63 {
            0o02 => tlb::tlbwi(cpu, pc),
            0o30 => misc::eret(cpu, pc),
            func => todo!("CPU COP0 Function '{:02o}' at {:08X}", func, pc),
        },
        opcode => todo!("CPU COP0 Opcode '{:02o}' at {:08X}", opcode, pc),
    }
}
