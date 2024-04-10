use super::{Core, Cp2, DfState, Flags, Vector};

mod compute;
mod load;
mod select;
mod single_lane;
mod store;

pub fn cop2(core: &mut Core, pc: u32, word: u32) -> DfState {
    match (word >> 21) & 31 {
        //0o00 => store::mfc2(core, pc, word),
        0o02 => store::cfc2(core, pc, word),
        //0o04 => load::mtc2(core, pc, word),
        //0o06 => load::ctc2(core, pc, word),
        0o20..=0o37 => match word & 63 {
            0x00 => compute::compute::<compute::VMulf>(core, pc, word),
            0x01 => compute::compute::<compute::VMulu>(core, pc, word),
            0x04 => compute::compute::<compute::VMudl>(core, pc, word),
            0x05 => compute::compute::<compute::VMudm>(core, pc, word),
            0x06 => compute::compute::<compute::VMudn>(core, pc, word),
            0x07 => compute::compute::<compute::VMudh>(core, pc, word),
            0x08 => compute::compute::<compute::VMacf>(core, pc, word),
            0x09 => compute::compute::<compute::VMacu>(core, pc, word),
            0x0c => compute::compute::<compute::VMadl>(core, pc, word),
            0x0d => compute::compute::<compute::VMadm>(core, pc, word),
            0x0e => compute::compute::<compute::VMadn>(core, pc, word),
            0x0f => compute::compute::<compute::VMadh>(core, pc, word),
            0x10 => compute::compute::<compute::VAdd>(core, pc, word),
            0x11 => compute::compute::<compute::VSub>(core, pc, word),
            0x13 => compute::compute::<compute::VAbs>(core, pc, word),
            0x14 => compute::compute::<compute::VAddc>(core, pc, word),
            0x15 => compute::compute::<compute::VSubc>(core, pc, word),
            0x1d => compute::vsar(core, pc, word),
            0x20 => compute::compute::<select::VLt>(core, pc, word),
            0x21 => compute::compute::<select::VEq>(core, pc, word),
            0x22 => compute::compute::<select::VNe>(core, pc, word),
            0x23 => compute::compute::<select::VGe>(core, pc, word),
            0x24 => compute::compute::<select::VCl>(core, pc, word),
            0x25 => compute::compute::<select::VCh>(core, pc, word),
            0x26 => compute::compute::<select::VCr>(core, pc, word),
            0x27 => compute::compute::<select::VMrg>(core, pc, word),
            0x28 => compute::compute::<compute::VAnd>(core, pc, word),
            0x29 => compute::compute::<compute::VNand>(core, pc, word),
            0x2a => compute::compute::<compute::VOr>(core, pc, word),
            0x2b => compute::compute::<compute::VNor>(core, pc, word),
            0x2c => compute::compute::<compute::VXor>(core, pc, word),
            0x2d => compute::compute::<compute::VNxor>(core, pc, word),
            0x30 => single_lane::single_lane::<single_lane::VRcp>(core, pc, word),
            0x31 => single_lane::single_lane::<single_lane::VRcpl>(core, pc, word),
            0x32 => single_lane::single_lane::<single_lane::VRcph>(core, pc, word),
            0x33 => single_lane::single_lane::<single_lane::VMov>(core, pc, word),
            0x34 => single_lane::single_lane::<single_lane::VRsq>(core, pc, word),
            0x35 => single_lane::single_lane::<single_lane::VRsql>(core, pc, word),
            0x36 => single_lane::single_lane::<single_lane::VRsqh>(core, pc, word),
            0x37 => single_lane::vnop(core, pc),
            0x3f => single_lane::vnull(core, pc),
            opcode => unimplemented!("RSP COP2 Function {:#04X} [PC:{:08X}]", opcode, core.pc()),
        },
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
