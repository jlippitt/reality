use super::{Cpu, DcState};
use crate::cpu::cp0::Cp0Register;

pub fn execute(cpu: &mut Cpu, word: u32) {
    match word >> 26 {
        0o17 => lui(cpu, word),
        0o20 => cop0(cpu, word),
        opcode => todo!("CPU Opcode: '{:02o}' at {:08X}", opcode, cpu.pc_debug),
    }
}

fn cop0(cpu: &mut Cpu, word: u32) {
    match (word >> 21) & 31 {
        0o04 => mtc0(cpu, word),
        opcode => todo!("COP0 Opcode '{:02o}' at {:08X}", opcode, cpu.pc_debug),
    }
}

fn lui(cpu: &mut Cpu, word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let imm = (word & 0xffff) as i64;
    println!(
        "{:08X}: LUI {}, 0x{:04X}",
        cpu.pc_debug,
        Cpu::REG_NAMES[rt],
        imm
    );
    cpu.dc = DcState::RegWrite {
        reg: rt,
        value: imm << 16,
    };
}

fn mtc0(cpu: &mut Cpu, word: u32) {
    let rt = ((word >> 16) & 31) as usize;
    let rd = Cp0Register::from((word >> 11) & 31);
    println!(
        "{:08X}: MTC0 {}, {:?}",
        cpu.pc_debug,
        Cpu::REG_NAMES[rt],
        rd
    );
    cpu.dc = DcState::Cp0Write {
        reg: rd,
        value: cpu.regs[rt],
    };
}
