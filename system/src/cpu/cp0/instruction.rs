use super::regs;
use super::Cp0;
use super::Cpu;
use tracing::trace;

mod tlb;
mod transfer;

pub fn cop0(cpu: &mut Cpu) {
    match (cpu.opcode[0] >> 21) & 31 {
        0o00 => transfer::mfc0(cpu),
        0o01 => transfer::dmfc0(cpu),
        0o04 => transfer::mtc0(cpu),
        0o05 => transfer::dmtc0(cpu),
        0o20..=0o37 => match cpu.opcode[0] & 63 {
            0o01 => tlb::tlbr(cpu),
            0o02 => tlb::tlbwi(cpu),
            0o06 => tlb::tlbwr(cpu),
            0o10 => tlb::tlbp(cpu),
            0o30 => eret(cpu),
            func => todo!("CPU COP0 Function '{:02o}' at {:08X}", func, cpu.pc[0]),
        },
        opcode => todo!("CPU COP0 Opcode '{:02o}' at {:08X}", opcode, cpu.pc[0]),
    }
}

fn eret(cpu: &mut Cpu) {
    trace!("{:08X}: ERET", cpu.pc[0]);

    let regs = &mut cpu.cp0.regs;

    if regs.status.erl() {
        cpu.pc[2] = regs.error_epc as u32;
        regs.status.set_erl(false);
    } else {
        cpu.pc[2] = regs.epc as u32;
        regs.status.set_exl(false);
    }

    // Neutralise the delay slot
    cpu.opcode[1] = 0;
    cpu.delay[1] = false;
    cpu.pc[1] = cpu.pc[2];

    cpu.ll_bit = false;
    cpu.cp0.update_int_mask();
}
