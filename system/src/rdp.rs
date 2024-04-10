use crate::memory::{Size, WriteMask};
use regs::{Regs, Status};
use tracing::debug;

mod regs;

pub struct RdpShared {
    regs: Regs,
}

pub struct Rdp {
    shared: RdpShared,
}

impl Rdp {
    pub fn new() -> Self {
        Self {
            shared: RdpShared {
                regs: Regs::default(),
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
            0 => self.regs.start,
            1 => self.regs.end,
            2 => self.regs.current,
            3 => self.regs.status.into(),
            _ => todo!("RDP Command Register Read: {}", index),
        }
    }

    pub fn write_register(&mut self, index: usize, mask: WriteMask) {
        match index {
            0 => mask.write_reg("DPC_START", &mut self.regs.start),
            1 => {
                mask.write_reg("DPC_END", &mut self.regs.end);
                todo!("RDP DMA");
            }
            3 => {
                let status = &mut self.regs.status;
                let raw = mask.raw();

                mask.set_or_clear(status, Status::set_xbus, 1, 0);
                mask.set_or_clear(status, Status::set_freeze, 3, 2);
                mask.set_or_clear(status, Status::set_flush, 5, 4);

                if (raw & 0x0040) != 0 {
                    status.set_tmem_busy(false)
                }

                if (raw & 0x0080) != 0 {
                    status.set_pipe_busy(false)
                }

                if (raw & 0x0100) != 0 {
                    status.set_buf_busy(false)
                }

                if (raw & 0x0200) != 0 {
                    todo!("RDP clock");
                }

                debug!("DPC_STATUS: {:?}", status);

                if status.freeze() {
                    todo!("RDP DMA freeze");
                }

                if status.flush() {
                    todo!("RDP DMA flush");
                }
            }
            _ => todo!(
                "RDP Command Register Write: {} <= {:08X}",
                index,
                mask.raw()
            ),
        }
    }
}
