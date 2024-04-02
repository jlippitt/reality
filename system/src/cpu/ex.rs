use super::cp0::Cp0;
use super::{Cpu, DcState};

mod arithmetic;
mod bitwise;
mod compare;
mod control;
mod load;
mod mul_div;
mod shift;
mod store;

pub fn execute(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match word >> 26 {
        0o00 => special(cpu, pc, word),
        0o01 => regimm(cpu, pc, word),
        0o02 => control::j::<false>(cpu, pc, word),
        0o03 => control::j::<true>(cpu, pc, word),
        0o04 => control::beq::<false>(cpu, pc, word),
        0o05 => control::bne::<false>(cpu, pc, word),
        0o06 => control::blez::<false>(cpu, pc, word),
        0o07 => control::bgtz::<false>(cpu, pc, word),
        0o10 => arithmetic::i_type_checked::<arithmetic::Add>(cpu, pc, word),
        0o11 => arithmetic::i_type_unchecked::<arithmetic::Add>(cpu, pc, word),
        0o12 => compare::slti(cpu, pc, word),
        0o13 => compare::sltiu(cpu, pc, word),
        0o14 => bitwise::i_type::<bitwise::And>(cpu, pc, word),
        0o15 => bitwise::i_type::<bitwise::Or>(cpu, pc, word),
        0o16 => bitwise::i_type::<bitwise::Xor>(cpu, pc, word),
        0o17 => load::lui(cpu, pc, word),
        0o20 => Cp0::cop0(cpu, pc, word),
        0o24 => control::beq::<true>(cpu, pc, word),
        0o25 => control::bne::<true>(cpu, pc, word),
        0o26 => control::blez::<true>(cpu, pc, word),
        0o27 => control::bgtz::<true>(cpu, pc, word),
        0o30 => arithmetic::i_type_checked::<arithmetic::Dadd>(cpu, pc, word),
        0o31 => arithmetic::i_type_unchecked::<arithmetic::Dadd>(cpu, pc, word),
        0o40 => load::load::<load::Lb>(cpu, pc, word),
        0o41 => load::load::<load::Lh>(cpu, pc, word),
        0o43 => load::load::<load::Lw>(cpu, pc, word),
        0o44 => load::load::<load::Lbu>(cpu, pc, word),
        0o45 => load::load::<load::Lhu>(cpu, pc, word),
        0o47 => load::load::<load::Lwu>(cpu, pc, word),
        0o50 => store::store::<store::Sb>(cpu, pc, word),
        0o51 => store::store::<store::Sh>(cpu, pc, word),
        0o53 => store::store::<store::Sw>(cpu, pc, word),
        0o57 => Cp0::cache(cpu, pc, word),
        0o67 => load::load::<load::Ld>(cpu, pc, word),
        0o77 => store::store::<store::Sd>(cpu, pc, word),
        opcode => todo!("CPU Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn special(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match word & 63 {
        0o00 => shift::fixed::<shift::Sll>(cpu, pc, word),
        0o02 => shift::fixed::<shift::Srl>(cpu, pc, word),
        0o03 => shift::fixed::<shift::Sra>(cpu, pc, word),
        0o04 => shift::variable::<shift::Sll>(cpu, pc, word),
        0o06 => shift::variable::<shift::Srl>(cpu, pc, word),
        0o07 => shift::variable::<shift::Sra>(cpu, pc, word),
        0o10 => control::jr::<false>(cpu, pc, word),
        0o11 => control::jr::<true>(cpu, pc, word),
        0o20 => mul_div::mfhi(cpu, pc, word),
        0o22 => mul_div::mflo(cpu, pc, word),
        0o30 => mul_div::mult(cpu, pc, word),
        0o31 => mul_div::multu(cpu, pc, word),
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
        0o54 => arithmetic::r_type_checked::<arithmetic::Dadd>(cpu, pc, word),
        0o55 => arithmetic::r_type_unchecked::<arithmetic::Dadd>(cpu, pc, word),
        0o56 => arithmetic::r_type_checked::<arithmetic::Dsub>(cpu, pc, word),
        0o57 => arithmetic::r_type_unchecked::<arithmetic::Dsub>(cpu, pc, word),
        opcode => todo!("CPU Special Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}

pub fn regimm(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    match (word >> 16) & 31 {
        0o00 => control::bltz::<false, false>(cpu, pc, word),
        0o01 => control::bgez::<false, false>(cpu, pc, word),
        0o02 => control::bltz::<false, true>(cpu, pc, word),
        0o03 => control::bgez::<false, true>(cpu, pc, word),
        0o20 => control::bltz::<true, false>(cpu, pc, word),
        0o21 => control::bgez::<true, false>(cpu, pc, word),
        0o22 => control::bltz::<true, true>(cpu, pc, word),
        0o23 => control::bgez::<true, true>(cpu, pc, word),
        opcode => todo!("CPU RegImm Opcode: '{:02o}' at {:08X}", opcode, pc),
    }
}
