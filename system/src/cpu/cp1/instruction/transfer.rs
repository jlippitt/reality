use super::cp0;
use super::{Bus, Cp1, Cpu, Format};
use tracing::trace;

pub fn mfc1(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!("{:08X}: MFC1 {}, F{}", cpu.pc[0], Cpu::REG_NAMES[rt], rd,);

    cpu.set_reg(rt, i32::cp1_reg(cpu, rd) as i64);
}

pub fn dmfc1(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!("{:08X}: DMFC1 {}, F{}", cpu.pc[0], Cpu::REG_NAMES[rt], rd,);

    cpu.set_reg(rt, i64::cp1_reg(cpu, rd));
}

pub fn mtc1(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!("{:08X}: MTC1 {}, F{}", cpu.pc[0], Cpu::REG_NAMES[rt], rd,);

    i32::set_cp1_reg(cpu, rd, cpu.regs[rt] as i32)
}

pub fn dmtc1(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!("{:08X}: DMTC1 {}, F{}", cpu.pc[0], Cpu::REG_NAMES[rt], rd,);

    i64::set_cp1_reg(cpu, rd, cpu.regs[rt])
}

pub fn cfc1(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: CFC1 {}, {:?}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cp1::CONTROL_REG_NAMES[rd]
    );

    cpu.set_reg(rt, cpu.cp1.read_control_reg(rd) as i64);
}

pub fn ctc1(cpu: &mut Cpu) {
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let rd = ((cpu.opcode[0] >> 11) & 31) as usize;

    trace!(
        "{:08X}: CTC1 {}, {:?}",
        cpu.pc[0],
        Cpu::REG_NAMES[rt],
        Cp1::CONTROL_REG_NAMES[rd]
    );

    cpu.cp1.write_control_reg(rd, cpu.regs[rt] as u32);
}

pub fn lwc1(cpu: &mut Cpu, bus: &mut impl Bus) {
    if !cpu.cp0.cp1_usable() {
        cp0::except(cpu, cp0::Exception::CoprocessorUnusable(1));
        return;
    }

    let base = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let offset = (cpu.opcode[0] & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: LWC1 F{}, {}({})",
        cpu.pc[0],
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    let address = cpu.regs[base].wrapping_add(offset) as u32;
    assert!((address & 3) == 0);

    if let Some(value) = cpu.read_data::<u32>(bus, address) {
        trace!("  [{:08X} => {:08X}]", address, value);
        i32::set_cp1_reg(cpu, rt, value as i32);
    }
}

pub fn ldc1(cpu: &mut Cpu, bus: &mut impl Bus) {
    if !cpu.cp0.cp1_usable() {
        cp0::except(cpu, cp0::Exception::CoprocessorUnusable(1));
        return;
    }

    let base = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let offset = (cpu.opcode[0] & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: LDC1 F{}, {}({})",
        cpu.pc[0],
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    let address = cpu.regs[base].wrapping_add(offset) as u32;
    assert!((address & 7) == 0);

    if let Some(value) = cpu.read_data::<u64>(bus, address) {
        trace!("  [{:08X} => {:016X}]", address, value);
        i64::set_cp1_reg(cpu, rt, value as i64);
    }
}

pub fn swc1(cpu: &mut Cpu, bus: &mut impl Bus) {
    if !cpu.cp0.cp1_usable() {
        cp0::except(cpu, cp0::Exception::CoprocessorUnusable(1));
        return;
    }

    let base = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let offset = (cpu.opcode[0] & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: SWC1 F{}, {}({})",
        cpu.pc[0],
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    let address = cpu.regs[base].wrapping_add(offset) as u32;
    let value = i32::cp1_reg(cpu, rt) as u32;
    assert!((address & 3) == 0);
    trace!("  [{:08X} <= {:08X}]", address, value);
    cpu.write_data(bus, address, value);
}

pub fn sdc1(cpu: &mut Cpu, bus: &mut impl Bus) {
    if !cpu.cp0.cp1_usable() {
        cp0::except(cpu, cp0::Exception::CoprocessorUnusable(1));
        return;
    }

    let base = ((cpu.opcode[0] >> 21) & 31) as usize;
    let rt = ((cpu.opcode[0] >> 16) & 31) as usize;
    let offset = (cpu.opcode[0] & 0xffff) as i16 as i64;

    trace!(
        "{:08X}: SDC1 F{}, {}({})",
        cpu.pc[0],
        rt,
        offset,
        Cpu::REG_NAMES[base],
    );

    let addr = cpu.regs[base].wrapping_add(offset) as u32;
    let value = i64::cp1_reg(cpu, rt) as u64;
    assert!((addr & 7) == 0);
    trace!("  [{:08X} <= {:016X}]", addr, value);
    cpu.write_data(bus, addr, value);
}
