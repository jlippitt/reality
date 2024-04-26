use super::cp0;
use super::cp1;
use super::{Bus, Cp0, Cpu};

mod arithmetic;
mod bitwise;
mod compare;
mod control;
mod exception;
mod load;
mod misc;
mod mul_div;
mod shift;
mod store;

pub fn execute(cpu: &mut Cpu, bus: &mut impl Bus) {
    match cpu.opcode[0] >> 26 {
        0o00 => special(cpu),
        0o01 => regimm(cpu),
        0o02 => control::j::<false>(cpu),
        0o03 => control::j::<true>(cpu),
        0o04 => control::beq::<false>(cpu),
        0o05 => control::bne::<false>(cpu),
        0o06 => control::blez::<false>(cpu),
        0o07 => control::bgtz::<false>(cpu),
        0o10 => arithmetic::i_type_checked::<arithmetic::Add>(cpu),
        0o11 => arithmetic::i_type_unchecked::<arithmetic::Add>(cpu),
        0o12 => compare::slti(cpu),
        0o13 => compare::sltiu(cpu),
        0o14 => bitwise::i_type::<bitwise::And>(cpu),
        0o15 => bitwise::i_type::<bitwise::Or>(cpu),
        0o16 => bitwise::i_type::<bitwise::Xor>(cpu),
        0o17 => load::lui(cpu),
        0o20 => cp0::cop0(cpu),
        0o21 => cp1::cop1(cpu),
        0o22 => exception::cop2(cpu),
        0o24 => control::beq::<true>(cpu),
        0o25 => control::bne::<true>(cpu),
        0o26 => control::blez::<true>(cpu),
        0o27 => control::bgtz::<true>(cpu),
        0o30 => arithmetic::i_type_checked::<arithmetic::Dadd>(cpu),
        0o31 => arithmetic::i_type_unchecked::<arithmetic::Dadd>(cpu),
        0o32 => load::load::<load::Ldl>(cpu, bus),
        0o33 => load::load::<load::Ldr>(cpu, bus),
        0o40 => load::load::<load::Lb>(cpu, bus),
        0o41 => load::load::<load::Lh>(cpu, bus),
        0o42 => load::load::<load::Lwl>(cpu, bus),
        0o43 => load::load::<load::Lw>(cpu, bus),
        0o44 => load::load::<load::Lbu>(cpu, bus),
        0o45 => load::load::<load::Lhu>(cpu, bus),
        0o46 => load::load::<load::Lwr>(cpu, bus),
        0o47 => load::load::<load::Lwu>(cpu, bus),
        0o50 => store::store::<store::Sb>(cpu, bus),
        0o51 => store::store::<store::Sh>(cpu, bus),
        0o52 => store::store::<store::Swl>(cpu, bus),
        0o53 => store::store::<store::Sw>(cpu, bus),
        0o54 => store::store::<store::Sdl>(cpu, bus),
        0o55 => store::store::<store::Sdr>(cpu, bus),
        0o56 => store::store::<store::Swr>(cpu, bus),
        0o57 => misc::cache(cpu, bus),
        0o60 => load::load::<load::Ll>(cpu, bus),
        0o61 => cp1::lwc1(cpu, bus),
        0o62 => exception::cop2(cpu),
        0o64 => load::load::<load::Lld>(cpu, bus),
        0o65 => cp1::ldc1(cpu, bus),
        0o66 => exception::cop2(cpu),
        0o67 => load::load::<load::Ld>(cpu, bus),
        0o70 => store::store::<store::Sc>(cpu, bus),
        0o71 => cp1::swc1(cpu, bus),
        0o72 => exception::cop2(cpu),
        0o74 => store::store::<store::Scd>(cpu, bus),
        0o75 => cp1::sdc1(cpu, bus),
        0o76 => exception::cop2(cpu),
        0o77 => store::store::<store::Sd>(cpu, bus),
        opcode => todo!("CPU Opcode: '{:02o}' at {:08X}", opcode, cpu.pc[0]),
    }
}

pub fn special(cpu: &mut Cpu) {
    match cpu.opcode[0] & 63 {
        0o00 => shift::fixed::<shift::Sll>(cpu),
        0o02 => shift::fixed::<shift::Srl>(cpu),
        0o03 => shift::fixed::<shift::Sra>(cpu),
        0o04 => shift::variable::<shift::Sll>(cpu),
        0o06 => shift::variable::<shift::Srl>(cpu),
        0o07 => shift::variable::<shift::Sra>(cpu),
        0o10 => control::jr(cpu),
        0o11 => control::jalr(cpu),
        0o14 => exception::syscall(cpu),
        0o15 => exception::break_(cpu),
        0o17 => misc::sync(cpu),
        0o20 => mul_div::mfhi(cpu),
        0o21 => mul_div::mthi(cpu),
        0o22 => mul_div::mflo(cpu),
        0o23 => mul_div::mtlo(cpu),
        0o24 => shift::variable::<shift::Dsll>(cpu),
        0o26 => shift::variable::<shift::Dsrl>(cpu),
        0o27 => shift::variable::<shift::Dsra>(cpu),
        0o30 => mul_div::mul_div::<mul_div::Mult>(cpu),
        0o31 => mul_div::mul_div::<mul_div::Multu>(cpu),
        0o32 => mul_div::mul_div::<mul_div::Div>(cpu),
        0o33 => mul_div::mul_div::<mul_div::Divu>(cpu),
        0o34 => mul_div::mul_div::<mul_div::Dmult>(cpu),
        0o35 => mul_div::mul_div::<mul_div::Dmultu>(cpu),
        0o36 => mul_div::mul_div::<mul_div::Ddiv>(cpu),
        0o37 => mul_div::mul_div::<mul_div::Ddivu>(cpu),
        0o40 => arithmetic::r_type_checked::<arithmetic::Add>(cpu),
        0o41 => arithmetic::r_type_unchecked::<arithmetic::Add>(cpu),
        0o42 => arithmetic::r_type_checked::<arithmetic::Sub>(cpu),
        0o43 => arithmetic::r_type_unchecked::<arithmetic::Sub>(cpu),
        0o44 => bitwise::r_type::<bitwise::And>(cpu),
        0o45 => bitwise::r_type::<bitwise::Or>(cpu),
        0o46 => bitwise::r_type::<bitwise::Xor>(cpu),
        0o47 => bitwise::r_type::<bitwise::Nor>(cpu),
        0o52 => compare::slt(cpu),
        0o53 => compare::sltu(cpu),
        0o54 => arithmetic::r_type_checked::<arithmetic::Dadd>(cpu),
        0o55 => arithmetic::r_type_unchecked::<arithmetic::Dadd>(cpu),
        0o56 => arithmetic::r_type_checked::<arithmetic::Dsub>(cpu),
        0o57 => arithmetic::r_type_unchecked::<arithmetic::Dsub>(cpu),
        0o64 => exception::teq(cpu),
        0o66 => exception::tne(cpu),
        0o70 => shift::fixed::<shift::Dsll>(cpu),
        0o72 => shift::fixed::<shift::Dsrl>(cpu),
        0o73 => shift::fixed::<shift::Dsra>(cpu),
        0o74 => shift::fixed32::<shift::Dsll>(cpu),
        0o76 => shift::fixed32::<shift::Dsrl>(cpu),
        0o77 => shift::fixed32::<shift::Dsra>(cpu),
        opcode => todo!("CPU Special Opcode: '{:02o}' at {:08X}", opcode, cpu.pc[0]),
    }
}

pub fn regimm(cpu: &mut Cpu) {
    match (cpu.opcode[0] >> 16) & 31 {
        0o00 => control::bltz::<false, false>(cpu),
        0o01 => control::bgez::<false, false>(cpu),
        0o02 => control::bltz::<false, true>(cpu),
        0o03 => control::bgez::<false, true>(cpu),
        0o20 => control::bltz::<true, false>(cpu),
        0o21 => control::bgez::<true, false>(cpu),
        0o22 => control::bltz::<true, true>(cpu),
        0o23 => control::bgez::<true, true>(cpu),
        opcode => todo!("CPU RegImm Opcode: '{:02o}' at {:08X}", opcode, cpu.pc[0]),
    }
}
