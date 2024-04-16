use super::{Cpu, DcOperation};
use tracing::trace;

pub fn slti(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: SLTI {}, {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    DcOperation::RegWrite {
        reg: rt,
        value: (cpu.regs[rs] < imm) as i64,
    }
}

pub fn sltiu(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i16 as u64;

    trace!(
        "{:08X}: SLTIU {}, {}, {}",
        pc,
        Cpu::REG_NAMES[rt],
        Cpu::REG_NAMES[rs],
        imm
    );

    DcOperation::RegWrite {
        reg: rt,
        value: ((cpu.regs[rs] as u64) < imm) as i64,
    }
}

pub fn slt(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLT {}, {}, {}",
        pc,
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    DcOperation::RegWrite {
        reg: rd,
        value: (cpu.regs[rs] < cpu.regs[rt]) as i64,
    }
}

pub fn sltu(cpu: &mut Cpu, pc: u32, word: u32) -> DcOperation {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: SLTU {}, {}, {}",
        pc,
        Cpu::REG_NAMES[rd],
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
    );

    DcOperation::RegWrite {
        reg: rd,
        value: ((cpu.regs[rs] as u64) < (cpu.regs[rt] as u64)) as i64,
    }
}
