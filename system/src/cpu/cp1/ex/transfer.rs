use super::{Cp1, Cpu, DcState, Format};
use tracing::trace;

pub fn mfc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: MFC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    DcState::RegWrite {
        reg: rt,
        value: i32::cp1_reg(cpu, rd) as i64,
    }
}

pub fn dmfc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: DMFC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    DcState::RegWrite {
        reg: rt,
        value: i64::cp1_reg(cpu, rd),
    }
}

pub fn mtc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: MTC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    i32::set_cp1_reg(cpu, rd, cpu.regs[rt] as i32).into()
}

pub fn dmtc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!("{:08X}: DMTC1 {}, F{}", pc, Cpu::REG_NAMES[rt], rd,);

    i64::set_cp1_reg(cpu, rd, cpu.regs[rt]).into()
}

pub fn cfc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: CFC1 {}, {:?}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp1::CONTROL_REG_NAMES[rd]
    );

    DcState::RegWrite {
        reg: rt,
        value: cpu.cp1.read_control_reg(rd) as i64,
    }
}

pub fn ctc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let rt = ((word >> 16) & 31) as usize;
    let rd = ((word >> 11) & 31) as usize;

    trace!(
        "{:08X}: CTC1 {}, {:?}",
        pc,
        Cpu::REG_NAMES[rt],
        Cp1::CONTROL_REG_NAMES[rd]
    );

    DcState::Cp1ControlRegWrite {
        reg: rd,
        value: cpu.regs[rt] as u32,
    }
}

pub fn lwc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: LWC1 F{}, {}({})",
        pc,
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    DcState::Cp1LoadWord {
        reg: rt,
        addr: cpu.regs[base].wrapping_add(offset) as u32,
    }
}

pub fn ldc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: LDC1 F{}, {}({})",
        pc,
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    DcState::Cp1LoadDoubleword {
        reg: rt,
        addr: cpu.regs[base].wrapping_add(offset) as u32,
    }
}

pub fn swc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: SWC1 F{}, {}({})",
        pc,
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    DcState::StoreWord {
        value: i32::cp1_reg(cpu, rt) as u32,
        addr: cpu.regs[base].wrapping_add(offset) as u32,
    }
}

pub fn sdc1(cpu: &mut Cpu, pc: u32, word: u32) -> DcState {
    let base = ((word >> 21) & 31) as usize;
    let rt = ((word >> 16) & 31) as usize;
    let offset = (word & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: SDC1 F{}, {}({})",
        pc,
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    DcState::StoreDoubleword {
        value: i64::cp1_reg(cpu, rt) as u64,
        addr: cpu.regs[base].wrapping_add(offset) as u32,
    }
}
