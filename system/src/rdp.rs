use crate::memory::{Size, WriteMask};
use regs::Status;

mod regs;

pub struct RdpShared {
    status: Status,
}

pub struct Rdp {
    shared: RdpShared,
}

impl Rdp {
    pub fn new() -> Self {
        Self {
            shared: RdpShared {
                status: Status::new(),
            },
        }
    }

    pub fn shared(&mut self) -> &mut RdpShared {
        &mut self.shared
    }

    pub fn read_command<T: Size>(&self, address: u32) -> T {
        T::truncate_u32(self.shared.read_register(address as usize >> 2))
    }

    pub fn write_command<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);
        self.shared.write_register(address as usize >> 2, mask);
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

impl RdpShared {
    pub fn read_register(&self, index: usize) -> u32 {
        match index {
            3 => self.status.into(),
            _ => todo!("RDP Command Register Read: {}", index),
        }
    }

    pub fn write_register(&mut self, index: usize, mask: WriteMask) {
        todo!(
            "RDP Command Register Write: {} <= {:08X}",
            index,
            mask.raw()
        );
    }
}
