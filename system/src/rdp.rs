use crate::memory::{Size, WriteMask};
use regs::Status;

mod regs;

pub struct Rdp {
    status: Status,
}

impl Rdp {
    pub fn new() -> Self {
        Self {
            status: Status::new(),
        }
    }

    pub fn read_command<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            3 => self.status.into(),
            _ => todo!("RDP Command Register Read: {:08X}", address),
        })
    }

    pub fn write_command<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        todo!(
            "RDP Command Register Write: {:08X} <= {:08X}",
            address,
            mask.raw()
        );
    }

    pub fn read_span<T: Size>(&self, address: u32) -> T {
        todo!("RDP Span Register Read: {:08X}", address);
    }

    pub fn write_span<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        todo!(
            "RDP Span Register Write: {:08X} <= {:08X}",
            address,
            mask.raw()
        );
    }
}
