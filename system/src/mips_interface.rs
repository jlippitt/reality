use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use regs::{Mode, Regs};
use std::sync::Arc;
use tracing::debug;

mod regs;

pub struct MipsInterface {
    regs: Regs,
    rcp_int: Arc<RcpInterrupt>,
}

impl MipsInterface {
    pub fn new(rcp_int: Arc<RcpInterrupt>) -> Self {
        Self {
            regs: Regs::default(),
            rcp_int,
        }
    }

    pub fn is_upper(&self) -> bool {
        self.regs.mode.upper()
    }

    pub fn is_repeat(&self) -> bool {
        self.regs.mode.repeat()
    }

    pub fn clear_repeat(&mut self) {
        self.regs.mode.set_repeat(false);
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::truncate_u32(match address >> 2 {
            1 => 0x0202_0102,
            2 => self.rcp_int.status() as u32,
            3 => self.rcp_int.mask() as u32,
            _ => todo!("MI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write_partial(&mut self.regs.mode, 0x007f);
                mask.set_or_clear(&mut self.regs.mode, Mode::set_repeat, 8, 7);
                mask.set_or_clear(&mut self.regs.mode, Mode::set_ebus, 10, 9);
                mask.set_or_clear(&mut self.regs.mode, Mode::set_upper, 13, 12);
                debug!("MI_MODE: {:?}", self.regs.mode);

                assert!(
                    !self.regs.mode.repeat() || self.regs.mode.repeat_count() == 15,
                    "Unsupported repeat mode configuration"
                );

                assert!(!self.regs.mode.ebus(), "EBus mode not supported");

                if (mask.raw() & 0x0800) != 0 {
                    self.rcp_int.clear(RcpIntType::DP);
                }
            }
            2 => (), // MI_INTERRUPT is read-only
            3 => {
                let mut int_mask = RcpIntType::from_bits_truncate(self.rcp_int.mask());
                mask.set_or_clear_flag(&mut int_mask, RcpIntType::SP, 1, 0);
                mask.set_or_clear_flag(&mut int_mask, RcpIntType::SI, 3, 2);
                mask.set_or_clear_flag(&mut int_mask, RcpIntType::AI, 5, 4);
                mask.set_or_clear_flag(&mut int_mask, RcpIntType::VI, 7, 6);
                mask.set_or_clear_flag(&mut int_mask, RcpIntType::PI, 9, 8);
                mask.set_or_clear_flag(&mut int_mask, RcpIntType::DP, 11, 10);
                self.rcp_int.set_mask(int_mask);
            }
            _ => todo!("MI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
