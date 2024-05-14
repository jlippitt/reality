use crate::gfx::GfxContext;
use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Memory, Size, WriteMask};
use crate::rdram::Rdram;
use decoder::{Context, Decoder};
use regs::{Regs, Status};
use renderer::Renderer;
use tracing::{debug, error_span};

mod decoder;
mod regs;
mod renderer;

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

pub struct Rdp {
    shared: RdpShared,
    decoder: Decoder,
    renderer: Renderer,
    rcp_int: RcpInterrupt,
}

impl Rdp {
    pub fn new(rcp_int: RcpInterrupt, gfx: &GfxContext) -> Self {
        Self {
            shared: RdpShared {
                regs: Regs::default(),
                dma_active: Dma { start: 0, end: 0 },
                dma_pending: None,
            },
            decoder: Decoder::new(),
            renderer: Renderer::new(gfx),
            rcp_int,
        }
    }

    pub fn shared(&mut self) -> &mut RdpShared {
        &mut self.shared
    }

    pub fn sync(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        let _span = error_span!("rdp").entered();
        self.renderer.sync(gfx, rdram);
    }

    #[inline(always)]
    pub fn step_core(&mut self, rdram: &mut Rdram, gfx: &GfxContext) {
        self.shared.regs.clock = self.shared.regs.clock.wrapping_add(1);

        if !self.decoder.running() || self.shared.regs.status.freeze() {
            return;
        }

        self.step_core_inner(rdram, gfx);
    }

    fn step_core_inner(&mut self, rdram: &mut Rdram, gfx: &GfxContext) {
        let sync_full = {
            let _span = error_span!("rdp").entered();

            self.decoder.step(Context {
                renderer: &mut self.renderer,
                rdram,
                gfx,
            })
        };

        if sync_full {
            self.sync(gfx, rdram);
            self.rcp_int.raise(RcpIntType::DP);

            let status = &mut self.shared.regs.status;
            status.set_pipe_busy(false);
            status.set_start_gclk(false);
            debug!("DPC_STATUS: {:?}", status);
        }
    }

    #[inline(always)]
    pub fn step_dma(&mut self, rdram: &Rdram, rsp_mem: &Memory<u128>) {
        if self.shared.dma_active.start >= self.shared.dma_active.end {
            return;
        }

        self.step_dma_inner(rdram, rsp_mem);
    }

    fn step_dma_inner(&mut self, rdram: &Rdram, rsp_mem: &Memory<u128>) {
        let dma = &mut self.shared.dma_active;

        assert!((dma.start & 7) == 0);
        assert!((dma.end & 7) == 0);

        let block_len = ((dma.end >> 3) - (dma.start >> 3)).min(16);
        let mut current = dma.start;

        if self.shared.regs.status.xbus() {
            for _ in 0..block_len {
                let command: u64 = rsp_mem.read(current as usize & 0xfff);
                self.decoder.write_command(command);
                current = current.wrapping_add(8) & 0x00ff_fff8;
            }

            debug!(
                "RDP DMA: {} bytes read from DMEM {:04X}",
                block_len * 8,
                dma.start
            );
        } else {
            for _ in 0..block_len {
                let command: u64 = rdram.read_single(current as usize);
                self.decoder.write_command(command);
                current = current.wrapping_add(8) & 0x00ff_fff8;
            }

            debug!(
                "RDP DMA: {} bytes read from RDRAM {:08X}",
                block_len * 8,
                dma.start
            );
        }

        self.decoder.restart();

        dma.start = current;

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
            4 => self.regs.clock & 0x00ff_ffff,
            _ => todo!("RDP Command Register Read: {}", index),
        }
    }

    pub fn write_register(&mut self, index: usize, mask: WriteMask) {
        match index {
            0 => {
                let status = &mut self.regs.status;

                if !status.start_pending() {
                    mask.write_partial(&mut self.regs.start, 0x00ff_fff8);
                    debug!("DPC_START: {:08X}", self.regs.start);
                    status.set_start_pending(true);
                    debug!("DPC_STATUS: {:?}", status);
                }
            }
            1 => {
                mask.write_partial(&mut self.regs.end, 0x00ff_fff8);
                debug!("DPC_END: {:08X}", self.regs.end);

                let status = &mut self.regs.status;

                if status.start_pending() {
                    // New transfer
                    if self.dma_active.start >= self.dma_active.end {
                        status.set_start_pending(false);

                        self.dma_active = Dma {
                            start: self.regs.start,
                            end: self.regs.end,
                        };
                    } else if let Some(dma) = &mut self.dma_pending {
                        assert!(status.end_pending());
                        dma.end = self.regs.end;
                    } else {
                        assert!(!status.end_pending());
                        status.set_end_pending(true);

                        self.dma_pending = Some(Dma {
                            start: self.regs.start,
                            end: self.regs.end,
                        });
                    }
                } else {
                    // Incremental transfer
                    assert!(self.dma_pending.is_none());
                    self.dma_active.end = self.regs.end;
                }

                debug!("RDP DMA Active: {:08X?}", self.dma_active);
                debug!("RDP DMA Pending: {:08X?}", self.dma_pending);

                status.set_pipe_busy(true);
                status.set_start_gclk(true);
                debug!("DPC_STATUS: {:?}", status);
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
                    self.regs.clock = 0;
                }

                debug!("DPC_STATUS: {:?}", status);

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
