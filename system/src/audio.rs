use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use regs::Regs;
use tracing::{trace, warn};

const DAC_FREQUENCY: i32 = 48681812;

mod regs;

pub struct AudioInterface {
    regs: Regs,
    dac_counter: i32,
    dma_count: u32,
    sample_count: [i32; 2],
    rcp_int: RcpInterrupt,
}

impl AudioInterface {
    pub fn new(rcp_int: RcpInterrupt) -> Self {
        Self {
            regs: Regs::default(),
            dac_counter: 0,
            dma_count: 0,
            sample_count: [0; 2],
            rcp_int,
        }
    }

    pub fn step(&mut self) {
        if self.dma_count == 0 {
            return;
        }

        self.dac_counter -= 1;

        if self.dac_counter < 0 {
            self.dma_count -= 1;
            trace!("AI DMA Count: {}", self.dma_count);

            if self.dma_count > 0 {
                self.sample_count[0] = self.sample_count[1];
                self.start_dma();
            }
        }
    }

    fn start_dma(&mut self) {
        let frequency = DAC_FREQUENCY / (self.regs.dacrate.dacrate() as i32 + 1);
        self.dac_counter = (125000000 * self.sample_count[0]) / frequency;
        trace!("AI Counter: {}", self.dac_counter);
        self.rcp_int.raise(RcpIntType::AI);
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::from_u32(match address >> 2 {
            3 => {
                // TODO: 'BC' and 'WC' bits
                let mut value = 0x0110_0000;

                if self.regs.control.dma_enable() {
                    value |= 0x0200_0000;
                }

                if self.dma_count >= 1 {
                    value |= 0x4000_0000;

                    if self.dma_count >= 2 {
                        value |= 0x8000_0001;
                    }
                }

                value | ((self.dac_counter as u32 & 0x3fff) << 1)
            }
            _ => todo!("AI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg("AI_DRAM_ADDR", &mut self.regs.dram_addr),
            1 => {
                mask.write_reg("AI_LENGTH", &mut self.regs.length);

                if self.dma_count < 2 {
                    self.sample_count[self.dma_count as usize] =
                        self.regs.length.length() as i32 / 4;

                    self.dma_count += 1;
                    trace!("AI DMA Count: {}", self.dma_count);

                    if self.dma_count < 2 {
                        self.start_dma();
                    }
                }
            }
            2 => {
                mask.write_reg("AI_CONTROL", &mut self.regs.control);

                if self.regs.control.dma_enable() {
                    warn!("TODO: AI DMA Transfers");
                }
            }
            3 => self.rcp_int.clear(RcpIntType::AI),
            4 => mask.write_reg("AI_DACRATE", &mut self.regs.dacrate),
            5 => mask.write_reg("AI_BITRATE", &mut self.regs.bitrate),
            _ => todo!("AI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}
