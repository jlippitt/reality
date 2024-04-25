use super::{RCP_CLOCK_RATE, VIDEO_DAC_RATE};
use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use crate::rdram::Rdram;
use regs::Regs;
use std::sync::{Arc, Mutex, RwLock};
use tracing::{debug, trace};

mod regs;

pub trait AudioReceiver {
    fn queue_samples(&mut self, sample_rate: u32, samples: &[u16]);
}

#[derive(Debug)]
struct Dma {
    dram_addr: u32,
    len: u32,
}

pub struct AudioInterface {
    regs: Regs,
    cycles_remaining: u32,
    cycles_per_sample: u32,
    sample_rate: u32,
    dma_active: Option<Dma>,
    dma_pending: Option<Dma>,
    rcp_int: Arc<Mutex<RcpInterrupt>>,
}

impl AudioInterface {
    pub fn new(rcp_int: Arc<Mutex<RcpInterrupt>>) -> Self {
        let regs = Regs::default();

        let (cycles_per_sample, sample_rate) = calc_cycles_per_sample(regs.dacrate.dacrate());

        Self {
            regs,
            cycles_remaining: cycles_per_sample,
            cycles_per_sample,
            sample_rate,
            dma_active: None,
            dma_pending: None,
            rcp_int,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline(always)]
    pub fn step(&mut self, rdram: &RwLock<Rdram>, receiver: &mut impl AudioReceiver) {
        self.cycles_remaining -= 1;

        if self.cycles_remaining != 0 {
            return;
        }

        self.step_inner(rdram, receiver);
    }

    fn step_inner(&mut self, rdram: &RwLock<Rdram>, receiver: &mut impl AudioReceiver) {
        self.cycles_remaining = self.cycles_per_sample;

        if let Some(dma_active) = &mut self.dma_active {
            let reader = rdram.read().unwrap();
            let left = reader.read_single(dma_active.dram_addr as usize);
            let right = reader.read_single((dma_active.dram_addr + 2) as usize);
            receiver.queue_samples(self.sample_rate, &[left, right]);
            trace!("AI DMA: 4 bytes read from {:08X}", dma_active.dram_addr);

            dma_active.dram_addr = (dma_active.dram_addr + 4) & 0x00ff_ffff;
            dma_active.len -= 4;

            // If DMA length has reached zero, switch to the next DMA if there is one
            if dma_active.len == 0 {
                self.dma_active = self.dma_pending.take();
                trace!("AI DMA Active: {:08X?}", self.dma_active);

                if self.dma_active.is_some() {
                    trace!("AI DMA Pending: {:08X?}", self.dma_active);
                    self.rcp_int.lock().unwrap().raise(RcpIntType::AI);
                }
            }
        } else {
            receiver.queue_samples(self.sample_rate, &[0, 0]);
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        T::truncate_u32(match address >> 2 {
            1 => {
                if let Some(dma_active) = &self.dma_active {
                    dma_active.len
                } else {
                    0
                }
            }
            3 => {
                // TODO: 'BC' and 'WC' bits
                // TODO: DAC counter value
                let mut value = 0x0110_0000;

                if self.regs.control.dma_enable() {
                    value |= 0x0200_0000;
                }

                if self.dma_active.is_some() {
                    value |= 0x4000_0000;

                    if self.dma_pending.is_some() {
                        value |= 0x8000_0001;
                    }
                }

                value
            }
            _ => todo!("AI Register Read: {:08X}", address),
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);

        match address >> 2 {
            0 => mask.write_reg("AI_DRAM_ADDR", &mut self.regs.dram_addr),
            1 => {
                let len = mask.raw() & 0x0003_fff8;

                // Don't queue DMAs with length of zero
                if len == 0 {
                    return;
                }

                let dma = Dma {
                    dram_addr: self.regs.dram_addr.dram_addr(),
                    len,
                };

                if self.dma_active.is_none() {
                    self.dma_active = Some(dma);
                    trace!("AI DMA Active: {:08X?}", self.dma_active);
                    self.rcp_int.lock().unwrap().raise(RcpIntType::AI);
                } else if self.dma_pending.is_none() {
                    self.dma_pending = Some(dma);
                    trace!("AI DMA Pending: {:08X?}", self.dma_pending);
                } else {
                    panic!("AI DMA queue full");
                }
            }
            2 => mask.write_reg("AI_CONTROL", &mut self.regs.control),
            3 => self.rcp_int.lock().unwrap().clear(RcpIntType::AI),
            4 => {
                mask.write_reg("AI_DACRATE", &mut self.regs.dacrate);
                (self.cycles_per_sample, self.sample_rate) =
                    calc_cycles_per_sample(self.regs.dacrate.dacrate());
            }
            5 => mask.write_reg("AI_BITRATE", &mut self.regs.bitrate),
            _ => todo!("AI Register Write: {:08X} <= {:08X}", address, mask.raw()),
        }
    }
}

fn calc_cycles_per_sample(dacrate: u32) -> (u32, u32) {
    let sample_rate = VIDEO_DAC_RATE / (dacrate + 1) as f64;
    let cycles_per_sample = (RCP_CLOCK_RATE / sample_rate) as u32;
    debug!("AI Sample Rate: {}", sample_rate as u32);
    debug!("AI Cycles Per Sample: {}", cycles_per_sample);
    (cycles_per_sample, sample_rate as u32)
}
