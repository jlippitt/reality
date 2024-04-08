use crate::memory::Size;
use dc::DcState;
use tracing::trace;

mod dc;
mod ex;

#[derive(Default)]
struct RfState {
    pc: u32,
    word: u32,
}

#[derive(Default)]
struct ExState {
    pc: u32,
    word: u32,
}

enum WbOperation {
    //Cp0RegWrite { reg: usize, value: i32 },
}

struct WbState {
    reg: usize,
    value: i32,
    op: Option<WbOperation>,
}

pub trait Bus {
    fn read_opcode(&self, address: u32) -> u32;
    fn read_data<T: Size>(&self, address: u32) -> T;
    fn write_data<T: Size>(&mut self, address: u32, value: T);
}

pub struct Core {
    wb: WbState,
    dc: DcState,
    ex: ExState,
    rf: RfState,
    pc: u32,
    regs: [i32; 32],
}

impl Core {
    const REG_NAMES: [&'static str; 32] = [
        "ZERO", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5", "T6",
        "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1", "GP", "SP",
        "FP", "RA",
    ];

    pub fn new() -> Self {
        Self {
            rf: Default::default(),
            ex: Default::default(),
            dc: DcState::Nop,
            wb: WbState {
                reg: 0,
                value: 0,
                op: None,
            },
            pc: 0,
            regs: [0; 32],
        }
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn set_pc(&mut self, value: u32) {
        self.pc = value & 0x0fff;
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        // WB
        self.regs[self.wb.reg] = self.wb.value;
        self.regs[0] = 0;

        if self.wb.reg != 0 {
            trace!("  {}: {:08X}", Self::REG_NAMES[self.wb.reg], self.wb.value);
        }

        // if let Some(op) = &self.wb.op {
        //     match *op {
        //         WbOperation::Cp0RegWrite { reg, value } => {
        //             todo!("RSP CP0");
        //         }
        //     }
        // }

        // DC
        dc::execute(self, bus);

        // EX
        if self.ex.word != 0 {
            // Operand forwarding from DC stage
            let tmp = self.regs[self.wb.reg];
            self.regs[self.wb.reg] = self.wb.value;
            self.regs[0] = 0;
            self.dc = ex::execute(self, self.ex.pc, self.ex.word);
            self.regs[self.wb.reg] = tmp;
        } else {
            trace!("{:08X}: NOP", self.ex.pc);
            self.dc = DcState::Nop;
        }

        // RF
        self.ex = ExState {
            pc: self.rf.pc,
            word: self.rf.word,
        };

        // IC
        self.rf = RfState {
            pc: self.pc,
            word: bus.read_opcode(self.pc),
        };

        self.pc = self.pc.wrapping_add(4) & 0x0fff;
    }

    fn branch(&mut self, condition: bool, offset: i32) {
        if condition {
            trace!("Branch taken");
            self.pc = (self.ex.pc as i32).wrapping_add(offset + 4) as u32 & 0x0fff;
        } else {
            trace!("Branch not taken");
        }
    }
}
