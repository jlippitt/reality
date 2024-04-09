use super::{Core, DfState};

mod load;

pub fn lwc2(core: &mut Core, pc: u32, word: u32) -> DfState {
    match (word >> 11) & 0x1f {
        0x00 => load::load::<load::Lbv>(core, pc, word),
        0x01 => load::load::<load::Lsv>(core, pc, word),
        0x02 => load::load::<load::Llv>(core, pc, word),
        0x03 => load::load::<load::Ldv>(core, pc, word),
        0x04 => load::load::<load::Lqv>(core, pc, word),
        // 0x05 => load::load::<load::Lrv>(core, pc, word),
        // 0x06 => load::load::<load::Lpv>(core, pc, word),
        // 0x07 => load::load::<load::Luv>(core, pc, word),
        // 0x0b => load::load::<load::Ltv>(core, pc, word),
        opcode => unimplemented!("RSP LWC2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
    }
}
