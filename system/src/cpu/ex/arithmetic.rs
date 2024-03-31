use super::{Cpu, DcState};
use tracing::trace;

pub fn addi(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16;

    trace!(
        "{:08X}: ADDI {}, {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    let Some(result) = (cpu.regs[rs] as i32).checked_add(imm as i32) else {
        todo!("Overflow exception");
    };

    DcState::RegWrite {
        reg: rt,
        value: result as i64,
    }
}

pub fn addiu(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16;

    trace!(
        "{:08X}: ADDIU {}, {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    let result = (cpu.regs[rs] as i32).wrapping_add(imm as i32);

    DcState::RegWrite {
        reg: rt,
        value: result as i64,
    }
}
