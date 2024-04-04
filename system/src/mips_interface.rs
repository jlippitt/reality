use crate::memory::{Size, WriteMask};
use regs::{Mask, Mode};
use tracing::{debug, warn};

mod regs;

pub struct MipsInterface {
    mode: Mode,
    mask: Mask,
}

impl MipsInterface {
    pub fn new() -> Self {
        Self {
            mode: Mode::new(),
            mask: Mask::new(),
        }
    }

    pub fn is_upper(&self) -> bool {
        self.mode.upper()
    }

    pub fn is_repeat(&self) -> bool {
        self.mode.repeat()
    }

    pub fn clear_repeat(&mut self) {
        self.mode.set_repeat(false);
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            1 => 0x0202_0102,
            _ => todo!("MI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => {
                mask.write_partial(&mut self.mode, 0x007f);
                mask.set_or_clear(&mut self.mode, Mode::set_repeat, 8, 7);
                mask.set_or_clear(&mut self.mode, Mode::set_ebus, 10, 9);
                mask.set_or_clear(&mut self.mode, Mode::set_upper, 13, 12);
                debug!("MI_MODE: {:?}", self.mode);

                assert!(
                    !self.mode.repeat() || self.mode.repeat_count() == 15,
                    "Unsupported repeat mode configuration"
                );

                assert!(!self.mode.ebus(), "EBus mode not supported");

                if (mask.raw() & 0x0800) != 0 {
                    warn!("TODO: Acknowledge MI interrupt")
                }
            }
            3 => {
                mask.set_or_clear(&mut self.mask, Mask::set_sp, 1, 0);
                mask.set_or_clear(&mut self.mask, Mask::set_si, 3, 2);
                mask.set_or_clear(&mut self.mask, Mask::set_ai, 5, 4);
                mask.set_or_clear(&mut self.mask, Mask::set_vi, 7, 6);
                mask.set_or_clear(&mut self.mask, Mask::set_pi, 9, 8);
                mask.set_or_clear(&mut self.mask, Mask::set_dp, 11, 10);
                debug!("MI_MASK: {:?}", self.mask);
            }
            _ => todo!("MI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
