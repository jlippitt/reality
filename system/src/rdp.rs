use crate::gfx::GfxContext;
use crate::memory::{Size, WriteMask};
use crate::rdram::Rdram;
use core::Core;
use regs::{Regs, Status};
use tracing::debug;

mod core;
mod regs;

#[derive(Debug)]
struct Dma {
    start: u32,
    end: u32,
}

pub struct RdpShared {
    regs: Regs,
    dma_active: Dma,
    dma_pending: Option<Dma>,
}

struct BusState {
    running: bool,
    cmd_list: Vec<u8>,
}

struct Bus<'a> {
    rdp: &'a mut BusState,
    rdram: &'a mut Rdram,
    gfx: &'a GfxContext,
}

pub struct Rdp {
    shared: RdpShared,
    core: Core,
    bus_state: BusState,
}

impl Rdp {
    pub fn new() -> Self {
        Self {
            shared: RdpShared {
                regs: Regs::default(),
                dma_active: Dma { start: 0, end: 0 },
                dma_pending: None,
            },
            core: Core::new(),
            bus_state: BusState {
                running: false,
                cmd_list: vec![],
            },
        }
    }

    pub fn step_core(&mut self, rdram: &mut Rdram, gfx: &GfxContext) {
        if !self.bus_state.running {
            return;
        }

        self.core.step(&mut Bus {
            rdp: &mut self.bus_state,
            rdram,
            gfx,
        });
    }

    pub fn step_dma(&mut self, rdram: &Rdram) {
        let dma = &mut self.shared.dma_active;

        if dma.start >= dma.end {
            return;
        }

        let block_len = (dma.end - dma.start).min(128);

        let mut buf = [0u8; 128];
        let data = &mut buf[0..(block_len as usize)];

        rdram.read_block(dma.start as usize & 0x00ff_ffff, data);
        self.bus_state.cmd_list.extend_from_slice(data);
        self.bus_state.running = true;

        debug!("RSP DMA: {} bytes read from {:08X}", block_len, dma.start);

        dma.start = (dma.start + block_len) & 0x00ff_ffff;

        if dma.start < dma.end {
            return;
        }

        if let Some(dma_pending) = self.shared.dma_pending.take() {
            let status = &mut self.shared.regs.status;
            status.set_start_pending(false);
            status.set_end_pending(false);
            debug!("DPC_STATUS: {:?}", status);

            self.shared.dma_active = dma_pending;
            debug!("RSP DMA Active: {:08X?}", self.shared.dma_active);
            debug!("RSP DMA Pending: {:08X?}", self.shared.dma_pending);
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
            2 => self.dma_active.start,
            3 => self.regs.status.into(),
            _ => todo!("RDP Command Register Read: {}", index),
        }
    }

    pub fn write_register(&mut self, index: usize, mask: WriteMask) {
        match index {
            0 => {
                mask.write_reg_hex("DPC_START", &mut self.regs.start);

                let status = &mut self.regs.status;
                status.set_start_pending(true);
                debug!("DPC_STATUS: {:?}", status);
            }
            1 => {
                mask.write_reg_hex("DPC_END", &mut self.regs.end);

                let end = self.regs.end & 0x00ff_ffff;
                let status = &mut self.regs.status;

                if status.start_pending() {
                    // New transfer
                    if self.dma_active.start >= self.dma_active.end {
                        status.set_start_pending(false);
                        debug!("DPC_STATUS: {:?}", status);

                        self.dma_active = Dma {
                            start: self.regs.start & 0x00ff_ffff,
                            end,
                        };
                    } else if let Some(dma) = &mut self.dma_pending {
                        assert!(status.end_pending());
                        dma.end = end;
                    } else {
                        assert!(!status.end_pending());
                        status.set_end_pending(true);
                        debug!("DPC_STATUS: {:?}", status);

                        self.dma_pending = Some(Dma {
                            start: self.regs.start & 0x00ff_ffff,
                            end,
                        });
                    }
                } else {
                    // Incremental transfer
                    assert!(self.dma_pending.is_none());
                    self.dma_active.end = end;
                }

                debug!("RSP DMA Active: {:08X?}", self.dma_active);
                debug!("RSP DMA Pending: {:08X?}", self.dma_pending);
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

impl<'a> core::Bus for Bus<'a> {
    //
}
