use bytemuck::Pod;
use std::mem;

mod ex;

const COLD_RESET_VECTOR: u32 = 0xbfc0_0000;

enum DcState {
    RegWrite { reg: usize, value: i64 },
    Nop,
}

struct WbState {
    reg: usize,
    value: i64,
}

pub trait Bus {
    fn read_single<T: Pod>(&self, address: u32) -> T;
}

pub struct Cpu {
    regs: [i64; 32],
    pc: u32,
    // Pipeline stages
    rf: Option<u32>,
    ex: Option<u32>,
    dc: DcState,
    wb: WbState,
}

impl Cpu {
    const REG_NAMES: [&'static str; 32] = [
        "ZERO", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5", "T6",
        "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1", "GP", "SP",
        "FP", "RA",
    ];

    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: COLD_RESET_VECTOR,
            rf: None,
            ex: None,
            dc: DcState::Nop,
            wb: WbState { reg: 0, value: 0 },
        }
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        // WB
        self.regs[self.wb.reg] = self.wb.value;
        self.regs[0] = 0;

        // DC
        match self.dc {
            DcState::RegWrite { reg, value } => {
                self.dc = DcState::Nop;
                self.wb.reg = reg;
                self.wb.value = value;
            }
            DcState::Nop => (),
        }

        // EX
        if let Some(word) = self.ex.take() {
            // Operand forwarding from DC stage
            let tmp = self.regs[self.wb.reg];
            self.regs[self.wb.reg] = self.wb.value;
            self.regs[0] = 0;
            ex::execute(self, word);
            self.regs[self.wb.reg] = tmp;

            if let DcState::RegWrite { reg, value } = self.dc {
                println!("  {}: {:016X}", Self::REG_NAMES[reg], value);
            }
        };

        // RF
        if let Some(word) = self.rf.take() {
            self.pc = self.pc.wrapping_add(4);
            self.ex = Some(word);
        };

        // IC
        self.rf = Some(self.read(bus, self.pc));
    }

    fn read<T: Pod>(&self, bus: &mut impl Bus, address: u32) -> T {
        let segment = address >> 29;

        if (segment & 6) != 4 {
            todo!("TLB lookups");
        }

        if segment == 4 {
            todo!("Cached reads");
        }

        if mem::size_of::<T>() > 4 {
            todo!("Block reads");
        }

        bus.read_single(address & 0x1fff_ffff)
    }
}
