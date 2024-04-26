use super::cp0;
use super::cp2;
use super::{Bus, Core};

mod arithmetic;
mod bitwise;
mod compare;
mod control;
mod exception;
mod load;
mod shift;
mod store;

pub fn execute(core: &mut Core, bus: &mut impl Bus) {
    match core.opcode[0] >> 26 {
        0o00 => special(core),
        0o01 => regimm(core),
        0o02 => control::j::<false>(core),
        0o03 => control::j::<true>(core),
        0o04 => control::beq(core),
        0o05 => control::bne(core),
        0o06 => control::blez(core),
        0o07 => control::bgtz(core),
        0o10 => arithmetic::i_type::<arithmetic::Add, false>(core),
        0o11 => arithmetic::i_type::<arithmetic::Add, true>(core),
        0o12 => compare::slti(core),
        0o13 => compare::sltiu(core),
        0o14 => bitwise::i_type::<bitwise::And>(core),
        0o15 => bitwise::i_type::<bitwise::Or>(core),
        0o16 => bitwise::i_type::<bitwise::Xor>(core),
        0o17 => load::lui(core),
        0o20 => cp0::cop0(core, bus),
        0o22 => cp2::cop2(core),
        0o24 => control::beq(core),
        0o25 => control::bne(core),
        0o26 => control::blez(core),
        0o27 => control::bgtz(core),
        0o40 => load::load::<load::Lb>(core, bus),
        0o41 => load::load::<load::Lh>(core, bus),
        0o43 => load::load::<load::Lw>(core, bus),
        0o44 => load::load::<load::Lbu>(core, bus),
        0o45 => load::load::<load::Lhu>(core, bus),
        0o47 => load::load::<load::Lwu>(core, bus),
        0o50 => store::store::<store::Sb>(core, bus),
        0o51 => store::store::<store::Sh>(core, bus),
        0o53 => store::store::<store::Sw>(core, bus),
        0o62 => cp2::lwc2(core, bus),
        0o72 => cp2::swc2(core, bus),
        opcode => todo!("core Opcode: '{:02o}' at {:08X}", opcode, core.pc[0]),
    }
}

pub fn special(core: &mut Core) {
    match core.opcode[0] & 63 {
        0o00 => shift::fixed::<shift::Sll>(core),
        0o02 => shift::fixed::<shift::Srl>(core),
        0o03 => shift::fixed::<shift::Sra>(core),
        0o04 => shift::variable::<shift::Sll>(core),
        0o06 => shift::variable::<shift::Srl>(core),
        0o07 => shift::variable::<shift::Sra>(core),
        0o10 => control::jr(core),
        0o11 => control::jalr(core),
        0o15 => exception::break_(core),
        0o40 => arithmetic::r_type::<arithmetic::Add, false>(core),
        0o41 => arithmetic::r_type::<arithmetic::Add, true>(core),
        0o42 => arithmetic::r_type::<arithmetic::Sub, false>(core),
        0o43 => arithmetic::r_type::<arithmetic::Sub, true>(core),
        0o44 => bitwise::r_type::<bitwise::And>(core),
        0o45 => bitwise::r_type::<bitwise::Or>(core),
        0o46 => bitwise::r_type::<bitwise::Xor>(core),
        0o47 => bitwise::r_type::<bitwise::Nor>(core),
        0o52 => compare::slt(core),
        0o53 => compare::sltu(core),
        opcode => todo!(
            "core Special Opcode: '{:02o}' at {:08X}",
            opcode,
            core.pc[0]
        ),
    }
}

pub fn regimm(core: &mut Core) {
    match (core.opcode[0] >> 16) & 31 {
        0o00 => control::bltz::<false>(core),
        0o01 => control::bgez::<false>(core),
        0o02 => control::bltz::<false>(core),
        0o03 => control::bgez::<false>(core),
        0o20 => control::bltz::<true>(core),
        0o21 => control::bgez::<true>(core),
        0o22 => control::bltz::<true>(core),
        0o23 => control::bgez::<true>(core),
        opcode => todo!("core RegImm Opcode: '{:02o}' at {:08X}", opcode, core.pc[0]),
    }
}
