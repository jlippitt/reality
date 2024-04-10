use super::cp0;
use super::cp2;
use super::{Core, DfState};

mod arithmetic;
mod bitwise;
mod compare;
mod control;
mod exception;
mod load;
mod shift;
mod store;

pub fn execute(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    match word >> 26 {
        0o00 => special(cpu, pc, word),
        0o01 => regimm(cpu, pc, word),
        0o02 => control::j::<false>(cpu, pc, word),
        0o03 => control::j::<true>(cpu, pc, word),
        0o04 => control::beq(cpu, pc, word),
        0o05 => control::bne(cpu, pc, word),
        0o06 => control::blez(cpu, pc, word),
        0o07 => control::bgtz(cpu, pc, word),
        0o10 => arithmetic::i_type_checked::<arithmetic::Add>(cpu, pc, word),
        0o11 => arithmetic::i_type_unchecked::<arithmetic::Add>(cpu, pc, word),
        0o12 => compare::slti(cpu, pc, word),
        0o13 => compare::sltiu(cpu, pc, word),
        0o14 => bitwise::i_type::<bitwise::And>(cpu, pc, word),
        0o15 => bitwise::i_type::<bitwise::Or>(cpu, pc, word),
        0o16 => bitwise::i_type::<bitwise::Xor>(cpu, pc, word),
        0o17 => load::lui(cpu, pc, word),
        0o20 => cp0::cop0(cpu, pc, word),
        0o22 => cp2::cop2(cpu, pc, word),
        0o24 => control::beq(cpu, pc, word),
        0o25 => control::bne(cpu, pc, word),
        0o26 => control::blez(cpu, pc, word),
        0o27 => control::bgtz(cpu, pc, word),
        0o40 => load::load::<load::Lb>(cpu, pc, word),
        0o41 => load::load::<load::Lh>(cpu, pc, word),
        0o43 => load::load::<load::Lw>(cpu, pc, word),
        0o44 => load::load::<load::Lbu>(cpu, pc, word),
        0o45 => load::load::<load::Lhu>(cpu, pc, word),
        0o50 => store::store::<store::Sb>(cpu, pc, word),
        0o51 => store::store::<store::Sh>(cpu, pc, word),
        0o53 => store::store::<store::Sw>(cpu, pc, word),
        0o62 => cp2::lwc2(cpu, pc, word),
        0o72 => cp2::swc2(cpu, pc, word),
        opcode => todo!("CPU Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn special(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    match word & 63 {
        0o00 => shift::fixed::<shift::Sll>(cpu, pc, word),
        0o02 => shift::fixed::<shift::Srl>(cpu, pc, word),
        0o03 => shift::fixed::<shift::Sra>(cpu, pc, word),
        0o04 => shift::variable::<shift::Sll>(cpu, pc, word),
        0o06 => shift::variable::<shift::Srl>(cpu, pc, word),
        0o07 => shift::variable::<shift::Sra>(cpu, pc, word),
        0o10 => control::jr::<false>(cpu, pc, word),
        0o11 => control::jr::<true>(cpu, pc, word),
        0o15 => exception::break_(cpu, pc),
        0o40 => arithmetic::r_type_checked::<arithmetic::Add>(cpu, pc, word),
        0o41 => arithmetic::r_type_unchecked::<arithmetic::Add>(cpu, pc, word),
        0o42 => arithmetic::r_type_checked::<arithmetic::Sub>(cpu, pc, word),
        0o43 => arithmetic::r_type_unchecked::<arithmetic::Sub>(cpu, pc, word),
        0o44 => bitwise::r_type::<bitwise::And>(cpu, pc, word),
        0o45 => bitwise::r_type::<bitwise::Or>(cpu, pc, word),
        0o46 => bitwise::r_type::<bitwise::Xor>(cpu, pc, word),
        0o47 => bitwise::r_type::<bitwise::Nor>(cpu, pc, word),
        0o52 => compare::slt(cpu, pc, word),
        0o53 => compare::sltu(cpu, pc, word),
        opcode => todo!("CPU Special Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn regimm(cpu: &mut Core, pc: u32, word: u32) -> DfState {
    match (word >> 16) & 31 {
        0o00 => control::bltz::<false>(cpu, pc, word),
        0o01 => control::bgez::<false>(cpu, pc, word),
        0o02 => control::bltz::<false>(cpu, pc, word),
        0o03 => control::bgez::<false>(cpu, pc, word),
        0o20 => control::bltz::<true>(cpu, pc, word),
        0o21 => control::bgez::<true>(cpu, pc, word),
        0o22 => control::bltz::<true>(cpu, pc, word),
        0o23 => control::bgez::<true>(cpu, pc, word),
        opcode => todo!("CPU RegImm Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}
