use super::{Cpu, DcState};

pub fn beq<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    println!(
        "{:08X}: BEQ{} {}, {}, {}",
        pc,
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
        offset
    );

    branch::<LIKELY>(cpu, cpu.regs[rs] == cpu.regs[rt], offset)
}

pub fn bne<const LIKELY: bool>(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rs = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = ((word & 0xffff) as i16 as i64) << 2;

    println!(
        "{:08X}: BNE{} {}, {}, {}",
        pc,
        if LIKELY { "L" } else { "" },
        Cpu::REG_NAMES[rs],
        Cpu::REG_NAMES[rt],
        offset
    );

    branch::<LIKELY>(cpu, cpu.regs[rs] != cpu.regs[rt], offset)
}

fn branch<const LIKELY: bool>(cpu: &mut Cpu, condition: bool, offset: i64) -> DcState {
    if condition {
        println!("Branch taken");
        cpu.pc = (cpu.rf.pc as i64).wrapping_add(offset) as u32;
    } else {
        println!("Branch not taken");

        if LIKELY {
            cpu.rf.word = 0;
        }
    }

    DcState::Nop
}