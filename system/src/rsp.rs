use super::memory::{Memory, WriteMask};
use crate::cpu::Size;
use regs::Status;

mod regs;

const MEM_SIZE: usize = 8192;

pub struct Rsp {
    mem: Memory,
    status: Status,
}

impl Rsp {
    pub fn new() -> Self {
        Self {
            mem: Memory::new(MEM_SIZE),
            status: Status::new().with_halted(true),
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if (address as usize) < MEM_SIZE {
            self.mem.read(address)
        } else if address >= 0x0004_0000 {
            T::from_u32(self.read_register(address))
        } else {
            panic!("Read from unmapped RSP address: {:08X}", address);
        }
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        if (address as usize) < MEM_SIZE {
            self.mem.write(address, value);
        } else if address >= 0x0004_0000 {
            self.write_register(address, WriteMask::new(address, value));
        } else {
            panic!("Write to unmapped RSP address: {:08X}", address);
        }
    }

    fn read_register(&self, address: u32) -> u32 {
        match (address & 0xffff) >> 2 {
            4 => self.status.into(),
            6 => self.status.dma_busy() as u32,
            _ => todo!("RSP Register Read: {:08X}", address),
        }
    }

    fn write_register(&mut self, address: u32, mask: WriteMask) {
        match (address & 0xffff) >> 2 {
            4 => {
                let raw = mask.raw();

                if (raw & 0x0000_0002) != 0 {
                    self.status.set_broke(false);
                }

                if (raw & 0x0000_0008) != 0 {
                    // TODO: Acknowledge RSP Interrupt
                }

                if (raw & 0x0000_0010) != 0 {
                    todo!("Trigger RSP interrupt");
                }

                set_or_clr(&mut self.status, Status::set_halted, 1, 0, raw);
                set_or_clr(&mut self.status, Status::set_sstep, 6, 5, raw);
                set_or_clr(&mut self.status, Status::set_intbreak, 8, 7, raw);
                set_or_clr(&mut self.status, Status::set_sig0, 10, 9, raw);
                set_or_clr(&mut self.status, Status::set_sig1, 12, 11, raw);
                set_or_clr(&mut self.status, Status::set_sig2, 14, 13, raw);
                set_or_clr(&mut self.status, Status::set_sig3, 16, 15, raw);
                set_or_clr(&mut self.status, Status::set_sig4, 18, 17, raw);
                set_or_clr(&mut self.status, Status::set_sig5, 20, 19, raw);
                set_or_clr(&mut self.status, Status::set_sig6, 22, 21, raw);
                set_or_clr(&mut self.status, Status::set_sig7, 24, 23, raw);

                println!("SP_STATUS: {:?}", self.status);
            }
            _ => todo!("RSP Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}

fn set_or_clr<F>(status: &mut Status, setter: F, set_bit: u32, clr_bit: u32, word: u32)
where
    F: Fn(&mut Status, bool),
{
    let set = (word & (1 << set_bit)) != 0;
    let clr = (word & (1 << clr_bit)) != 0;

    match (set, clr) {
        (false, false) => (),
        (false, true) => setter(status, false),
        (true, false) => setter(status, true),
        (true, true) => panic!(
            "Conflict between SET_* and CLR_* bits {} and {}",
            set_bit, clr_bit
        ),
    }
}
