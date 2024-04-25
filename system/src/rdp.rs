use crate::gfx::GfxContext;
use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Size, WriteMask};
use crate::rdram::Rdram;
use crate::rsp::RspInterface;
use decoder::{Context, Decoder};
use regs::{Regs, Status};
use renderer::Renderer;
use std::sync::{Arc, Mutex, RwLock};
use tracing::{debug, warn};

mod decoder;
mod regs;
mod renderer;

#[derive(Debug)]
struct Dma {
    start: u32,
    end: u32,
}

pub struct RdpCore {
    decoder: Decoder,
    renderer: Renderer,
    rcp_int: Arc<RcpInterrupt>,
}

pub struct RdpInterface {
    regs: Regs,
    dma_active: Dma,
    dma_pending: Option<Dma>,
}

impl RdpCore {
    pub fn new(rcp_int: Arc<RcpInterrupt>, gfx: &GfxContext) -> Self {
        Self {
            decoder: Decoder::new(),
            renderer: Renderer::new(gfx),
            rcp_int,
        }
    }

    #[inline(always)]
    pub fn step_core(
        &mut self,
        iface: &Mutex<RdpInterface>,
        rsp_iface: &Mutex<RspInterface>,
        rdram: &RwLock<Rdram>,
        gfx: &GfxContext,
    ) {
        {
            let mut iface_lock = iface.lock().unwrap();

            iface_lock.step_dma(rdram, rsp_iface, &mut self.decoder);

            if !self.decoder.running() || iface_lock.regs.status.freeze() {
                return;
            }
        }

        self.step_core_inner(iface, rdram, gfx);
    }

    fn step_core_inner(
        &mut self,
        iface: &Mutex<RdpInterface>,
        rdram: &RwLock<Rdram>,
        gfx: &GfxContext,
    ) {
        let sync_full = {
            self.decoder.step(Context {
                renderer: &mut self.renderer,
                rdram,
                gfx,
            })
        };

        if sync_full {
            {
                self.renderer.sync(gfx, rdram);
            }

            self.rcp_int.raise(RcpIntType::DP);

            let mut iface_lock = iface.lock().unwrap();
            let status = &mut iface_lock.regs.status;
            status.set_pipe_busy(false);
            status.set_start_gclk(false);
            debug!("DPC_STATUS: {:?}", status);
        }
    }
}

impl RdpInterface {
    pub fn new() -> Self {
        Self {
            regs: Regs::default(),
            dma_active: Dma { start: 0, end: 0 },
            dma_pending: None,
        }
    }

    #[inline(always)]
    pub fn step_dma(
        &mut self,
        rdram: &RwLock<Rdram>,
        rsp_iface: &Mutex<RspInterface>,
        decoder: &mut Decoder,
    ) {
        if self.dma_active.start >= self.dma_active.end {
            return;
        }

        self.step_dma_inner(rdram, rsp_iface, decoder);
    }

    fn step_dma_inner(
        &mut self,
        rdram: &RwLock<Rdram>,
        rsp_iface: &Mutex<RspInterface>,
        decoder: &mut Decoder,
    ) {
        let dma = &mut self.dma_active;

        assert!((dma.start & 7) == 0);
        assert!((dma.end & 7) == 0);

        let block_len = ((dma.end >> 3) - (dma.start >> 3)).min(16);
        let mut current = dma.start;

        if self.regs.status.xbus() {
            let rsp_lock = rsp_iface.lock().unwrap();
            let rsp_mem = rsp_lock.mem();

            for _ in 0..block_len {
                let command: u64 = rsp_mem.read(current as usize & 0xfff);
                decoder.write_command(command);
                current = current.wrapping_add(8) & 0x00ff_fff8;
            }

            debug!(
                "RDP DMA: {} bytes read from DMEM {:04X}",
                block_len * 8,
                dma.start
            );
        } else {
            let reader = rdram.read().unwrap();

            for _ in 0..block_len {
                let command: u64 = reader.read_single(current as usize);
                decoder.write_command(command);
                current = current.wrapping_add(8) & 0x00ff_fff8;
            }

            debug!(
                "RDP DMA: {} bytes read from RDRAM {:08X}",
                block_len * 8,
                dma.start
            );
        }

        decoder.restart();

        dma.start = current;

        if dma.start < dma.end {
            return;
        }

        if let Some(dma_pending) = self.dma_pending.take() {
            let status = &mut self.regs.status;
            status.set_start_pending(false);
            status.set_end_pending(false);
            debug!("DPC_STATUS: {:?}", status);

            self.dma_active = dma_pending;
            debug!("RSP DMA Active: {:08X?}", self.dma_active);
            debug!("RSP DMA Pending: {:08X?}", self.dma_pending);
        }
    }

    pub fn read_command<T: Size>(&self, address: u32) -> T {
        T::truncate_u32(self.read_register(address as usize >> 2))
    }

    pub fn write_command<T: Size>(&mut self, address: u32, value: T) {
        let mask = WriteMask::new(address, value);
        self.write_register(address as usize >> 2, mask);
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
                    warn!("TODO: RDP clock");
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
