use super::{Core, DfState, Flags, Vector};

mod compute;
mod load;
mod store;

pub fn cop2(core: &mut Core, pc: u32, word: u32) -> DfState {
    match word & 31 {
        0x10 => compute::compute::<compute::VAdd>(core, pc, word),
        0x1d => compute::vsar(core, pc, word),
        opcode => unimplemented!("RSP COP2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
    }
}

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

pub fn swc2(core: &mut Core, pc: u32, word: u32) -> DfState {
    match (word >> 11) & 0x1f {
        0x00 => store::store::<store::Sbv>(core, pc, word),
        0x01 => store::store::<store::Ssv>(core, pc, word),
        0x02 => store::store::<store::Slv>(core, pc, word),
        0x03 => store::store::<store::Sdv>(core, pc, word),
        0x04 => store::store::<store::Sqv>(core, pc, word),
        // 0x05 => store::store::<store::Srv>(core, pc, word),
        // 0x06 => store::store::<store::Spv>(core, pc, word),
        // 0x07 => store::store::<store::Suv>(core, pc, word),
        // 0x0b => store::store::<store::Stv>(core, pc, word),
        opcode => unimplemented!("RSP SWC2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
    }
}
