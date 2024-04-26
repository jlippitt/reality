use super::{Bus, Core, Cp2, Flags, Vector};

mod compute;
mod load;
mod select;
mod single_lane;
mod store;

pub fn cop2(core: &mut Core) {
    match (core.opcode[0] >> 21) & 31 {
        0o00 => store::mfc2(core),
        0o02 => store::cfc2(core),
        0o04 => load::mtc2(core),
        0o06 => load::ctc2(core),
        //0o06 => load::ctc2(core),
        0o20..=0o37 => match core.opcode[0] & 63 {
            0x00 => compute::compute::<compute::VMulf>(core),
            0x01 => compute::compute::<compute::VMulu>(core),
            0x04 => compute::compute::<compute::VMudl>(core),
            0x05 => compute::compute::<compute::VMudm>(core),
            0x06 => compute::compute::<compute::VMudn>(core),
            0x07 => compute::compute::<compute::VMudh>(core),
            0x08 => compute::compute::<compute::VMacf>(core),
            0x09 => compute::compute::<compute::VMacu>(core),
            0x0c => compute::compute::<compute::VMadl>(core),
            0x0d => compute::compute::<compute::VMadm>(core),
            0x0e => compute::compute::<compute::VMadn>(core),
            0x0f => compute::compute::<compute::VMadh>(core),
            0x10 => compute::compute::<compute::VAdd>(core),
            0x11 => compute::compute::<compute::VSub>(core),
            0x13 => compute::compute::<compute::VAbs>(core),
            0x14 => compute::compute::<compute::VAddc>(core),
            0x15 => compute::compute::<compute::VSubc>(core),
            0x1d => compute::vsar(core),
            0x20 => compute::compute::<select::VLt>(core),
            0x21 => compute::compute::<select::VEq>(core),
            0x22 => compute::compute::<select::VNe>(core),
            0x23 => compute::compute::<select::VGe>(core),
            0x24 => compute::compute::<select::VCl>(core),
            0x25 => compute::compute::<select::VCh>(core),
            0x26 => compute::compute::<select::VCr>(core),
            0x27 => compute::compute::<select::VMrg>(core),
            0x28 => compute::compute::<compute::VAnd>(core),
            0x29 => compute::compute::<compute::VNand>(core),
            0x2a => compute::compute::<compute::VOr>(core),
            0x2b => compute::compute::<compute::VNor>(core),
            0x2c => compute::compute::<compute::VXor>(core),
            0x2d => compute::compute::<compute::VNxor>(core),
            0x30 => single_lane::single_lane::<single_lane::VRcp>(core),
            0x31 => single_lane::single_lane::<single_lane::VRcpl>(core),
            0x32 => single_lane::single_lane::<single_lane::VRcph>(core),
            0x33 => single_lane::single_lane::<single_lane::VMov>(core),
            0x34 => single_lane::single_lane::<single_lane::VRsq>(core),
            0x35 => single_lane::single_lane::<single_lane::VRsql>(core),
            0x36 => single_lane::single_lane::<single_lane::VRsqh>(core),
            0x37 => single_lane::vnop(core),
            0x3f => single_lane::vnull(core),
            opcode => unimplemented!("RSP COP2 Function {:#04X} [PC:{:08X}]", opcode, core.pc[0]),
        },
        opcode => unimplemented!("RSP COP2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc[0]),
    }
}

pub fn lwc2(core: &mut Core, bus: &impl Bus) {
    match (core.opcode[0] >> 11) & 0x1f {
        0x00 => load::load::<load::Lbv>(core, bus),
        0x01 => load::load::<load::Lsv>(core, bus),
        0x02 => load::load::<load::Llv>(core, bus),
        0x03 => load::load::<load::Ldv>(core, bus),
        0x04 => load::load::<load::Lqv>(core, bus),
        0x05 => load::load::<load::Lrv>(core, bus),
        0x06 => load::load::<load::Lpv>(core, bus),
        0x07 => load::load::<load::Luv>(core, bus),
        0x0b => load::load::<load::Ltv>(core, bus),
        opcode => unimplemented!("RSP LWC2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
    }
}

pub fn swc2(core: &Core, bus: &mut impl Bus) {
    match (core.opcode[0] >> 11) & 0x1f {
        0x00 => store::store::<store::Sbv>(core, bus),
        0x01 => store::store::<store::Ssv>(core, bus),
        0x02 => store::store::<store::Slv>(core, bus),
        0x03 => store::store::<store::Sdv>(core, bus),
        0x04 => store::store::<store::Sqv>(core, bus),
        0x05 => store::store::<store::Srv>(core, bus),
        0x06 => store::store::<store::Spv>(core, bus),
        0x07 => store::store::<store::Suv>(core, bus),
        0x0b => store::store::<store::Stv>(core, bus),
        opcode => unimplemented!("RSP SWC2 Opcode {:#04X} [PC:{:08X}]", opcode, core.pc()),
    }
}
