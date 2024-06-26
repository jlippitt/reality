use crate::interrupt::{RcpIntType, RcpInterrupt};
use crate::memory::{Memory, Size, WriteMask};
use crate::rdp::RdpShared;
use crate::rdram::Rdram;
use core::Core;
use regs::{DmaLength, DmaRamAddr, DmaSpAddr, Regs, Status};
use tracing::{debug, debug_span, trace};

mod core;
mod regs;

const MEM_SIZE: usize = 8192;

#[derive(Debug, Default)]
struct Dma {
    sp_addr: DmaSpAddr,
    ram_addr: DmaRamAddr,
    len: DmaLength,
    reload_len: u32,
    write: bool,
}

struct RspShared {
    mem: Memory<u128>,
    regs: Regs,
    dma_in_progress: bool,
    dma_active: Dma,
    dma_pending: Option<Dma>,
    rcp_int: RcpInterrupt,
}

#[cfg(feature = "profiling")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stats {
    pub instruction_cycles: u64,
    pub halt_cycles: u64,
}

struct Bus<'a> {
    rsp: &'a mut RspShared,
    rdp: &'a mut RdpShared,
}

pub struct Rsp {
    core: Core,
    shared: RspShared,
    #[cfg(feature = "profiling")]
    stats: Stats,
}

impl Rsp {
    pub fn new(rcp_int: RcpInterrupt, ipl3_data: Option<&[u8]>) -> Self {
        let mem = if let Some(ipl3_data) = ipl3_data {
            let mut vec = Vec::from(ipl3_data);
            vec.resize(MEM_SIZE, 0);
            Memory::from_bytes(&vec)
        } else {
            Memory::with_byte_len(MEM_SIZE)
        };

        Self {
            core: Core::new(),
            shared: RspShared {
                mem,
                dma_in_progress: false,
                dma_active: Dma::default(),
                dma_pending: None,
                regs: Regs::default(),
                rcp_int,
            },
            #[cfg(feature = "profiling")]
            stats: Stats::default(),
        }
    }

    pub fn mem(&self) -> &Memory<u128> {
        &self.shared.mem
    }

    #[cfg(feature = "profiling")]
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    #[cfg(feature = "profiling")]
    pub fn reset_stats(&mut self) {
        self.stats = Stats::default();
    }

    #[inline(always)]
    pub fn step_core(&mut self, rdp_shared: &mut RdpShared) {
        if self.shared.regs.status.halted() {
            #[cfg(feature = "profiling")]
            {
                self.stats.halt_cycles += 1;
            }

            return;
        }

        #[cfg(feature = "profiling")]
        {
            self.stats.instruction_cycles += 1;
        }

        self.step_core_inner(rdp_shared);
    }

    fn step_core_inner(&mut self, rdp_shared: &mut RdpShared) {
        {
            let _span = debug_span!("rsp").entered();

            self.core.step(&mut Bus {
                rsp: &mut self.shared,
                rdp: rdp_shared,
            });
        }

        let status = &mut self.shared.regs.status;
        status.set_halted(status.halted() | status.sstep());
    }

    #[inline(always)]
    pub fn step_dma(&mut self, rdram: &mut Rdram) {
        if !self.shared.dma_in_progress {
            return;
        }

        self.step_dma_inner(rdram);
    }

    fn step_dma_inner(&mut self, rdram: &mut Rdram) {
        let dma = &mut self.shared.dma_active;

        let bank_offset = (dma.sp_addr.mem_bank() as u32) << 12;
        let mem_addr = dma.sp_addr.mem_addr() & 0x0ff8;
        let dram_addr = dma.ram_addr.dram_addr() & 0x00ff_fff8;
        let row_len = (dma.len.len() + 8) & !7;
        let block_len = row_len.min(128);

        let mut buf = [0u8; 128];
        let data = &mut buf[0..(block_len as usize)];

        if dma.write {
            let mut byte_addr = mem_addr as usize;

            for byte in data.iter_mut() {
                *byte = self.shared.mem[bank_offset as usize + byte_addr];
                byte_addr = (byte_addr + 1) & 0x0fff;
            }

            rdram.write_block(dram_addr as usize, data);

            debug!(
                "RSP DMA: {} bytes written from {:08X} to {:08X}",
                block_len,
                bank_offset | mem_addr,
                dram_addr,
            );
        } else {
            rdram.read_block(dram_addr as usize, data);

            let mut byte_addr = mem_addr as usize;

            for byte in data.iter() {
                self.shared.mem[bank_offset as usize + byte_addr] = *byte;
                byte_addr = (byte_addr + 1) & 0x0fff;
            }

            debug!(
                "RSP DMA: {} bytes read from {:08X} to {:08X}",
                block_len,
                dram_addr,
                bank_offset | mem_addr,
            );
        }

        dma.sp_addr.set_mem_addr((mem_addr + block_len) & 0x0fff);

        if row_len == block_len {
            dma.ram_addr
                .set_dram_addr((dram_addr + block_len + dma.len.skip()) & 0x00ff_ffff);

            let count = dma.len.count();

            if count == 0 {
                if let Some(dma_pending) = self.shared.dma_pending.take() {
                    self.shared.dma_active = dma_pending;
                    trace!("RSP DMA Active: {:08X?}", self.shared.dma_active);
                    trace!("RSP DMA Pending: {:08X?}", self.shared.dma_pending);
                } else {
                    self.shared.dma_in_progress = false;
                    dma.len.set_len(0x0ff8);
                }

                return;
            }

            dma.len.set_len(dma.reload_len);
            dma.len.set_count(count - 1);
        } else {
            dma.ram_addr
                .set_dram_addr((dram_addr + block_len) & 0x00ff_ffff);
            dma.len.set_len(dma.len.len() - block_len);
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if (address as usize) < 0x0004_0000 {
            return self.shared.mem.read(address as usize & 0x0000_1fff);
        }

        T::truncate_u32(if (address & 0x0004_0000) == 0x0004_0000 {
            self.shared.read_register((address as usize & 0xffff) >> 2)
        } else if address == 0x0008_0000 {
            self.core.pc()
        } else {
            panic!("Read from unmapped RSP address: {:08X}", address);
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        if address < 0x0004_0000 {
            return self.shared.mem.write(address as usize & 0x0000_1fff, value);
        }

        let mask = WriteMask::new(address, value);

        if (address & 0x0004_0000) == 0x0004_0000 {
            self.shared
                .write_register((address as usize & 0xffff) >> 2, mask);
        } else if address == 0x0008_0000 {
            let mut pc = self.core.pc();
            mask.write(&mut pc);
            self.core.set_pc(pc);
            debug!("SP_PC: {:08X}", pc);
        } else {
            panic!("Write to unmapped RSP address: {:08X}", address);
        }
    }
}

impl RspShared {
    fn read_register(&self, index: usize) -> u32 {
        match index {
            0 => self.dma_active.sp_addr.into(),
            1 => self.dma_active.ram_addr.into(),
            2 | 3 => self.dma_active.len.into(),
            4 => self
                .regs
                .status
                .with_dma_busy(self.dma_in_progress)
                .with_dma_full(self.dma_pending.is_some())
                .into(),
            5 => self.dma_pending.is_some() as u32,
            6 => self.dma_in_progress as u32,
            7 => {
                let value = self.regs.semaphore.get();
                self.regs.semaphore.set(true);
                trace!("SP_SEMAPHORE: {}", self.regs.semaphore.get());
                value as u32
            }
            _ => todo!("RSP Register Read: {}", index),
        }
    }

    fn write_register(&mut self, index: usize, mask: WriteMask) {
        match index {
            0 => mask.write_reg_hex("SP_DMA_SPADDR", &mut self.regs.dma_sp_addr),
            1 => mask.write_reg_hex("SP_DMA_RAMADDR", &mut self.regs.dma_ram_addr),
            2 => self.enqueue_dma(mask.raw(), false),
            3 => self.enqueue_dma(mask.raw(), true),
            4 => {
                let status = &mut self.regs.status;
                let raw = mask.raw();

                if (raw & 0x0000_0004) != 0 {
                    status.set_broke(false);
                }

                match (raw >> 3) & 3 {
                    1 => self.rcp_int.clear(RcpIntType::SP),
                    2 => self.rcp_int.raise(RcpIntType::SP),
                    _ => (),
                }

                mask.set_or_clear(status, Status::set_halted, 1, 0);
                mask.set_or_clear(status, Status::set_sstep, 6, 5);
                mask.set_or_clear(status, Status::set_intbreak, 8, 7);
                mask.set_or_clear(status, Status::set_sig0, 10, 9);
                mask.set_or_clear(status, Status::set_sig1, 12, 11);
                mask.set_or_clear(status, Status::set_sig2, 14, 13);
                mask.set_or_clear(status, Status::set_sig3, 16, 15);
                mask.set_or_clear(status, Status::set_sig4, 18, 17);
                mask.set_or_clear(status, Status::set_sig5, 20, 19);
                mask.set_or_clear(status, Status::set_sig6, 22, 21);
                mask.set_or_clear(status, Status::set_sig7, 24, 23);

                debug!("SP_STATUS: {:?}", status);
            }
            7 => {
                self.regs.semaphore.set(false);
                trace!("SP_SEMAPHORE: {}", self.regs.semaphore.get());
            }
            _ => todo!("RSP Register Write: {} <= {:08X}", index, mask.raw()),
        }
    }

    fn enqueue_dma(&mut self, len: u32, write: bool) {
        let len = DmaLength::from(len);

        // Don't queue DMAs with length of zero
        if len.len() == 0 {
            return;
        }

        let dma = Dma {
            sp_addr: self.regs.dma_sp_addr,
            ram_addr: self.regs.dma_ram_addr,
            len,
            reload_len: len.len(),
            write,
        };

        if !self.dma_in_progress {
            self.dma_in_progress = true;
            self.dma_active = dma;
            trace!("RSP DMA Active: {:08X?}", self.dma_active);
        } else if self.dma_pending.is_none() {
            self.dma_pending = Some(dma);
            trace!("RSP DMA Pending: {:08X?}", self.dma_pending);
        } else {
            panic!("RSP DMA queue full");
        }
    }
}

impl<'a> core::Bus for Bus<'a> {
    fn read_opcode(&self, address: u32) -> u32 {
        let address = address as usize;
        debug_assert!(address < (MEM_SIZE / 2));
        debug_assert!((address & 3) == 0);
        self.rsp.mem.read(0x1000 | address)
    }

    fn read_data<T: Size>(&self, address: u32) -> T {
        self.rsp.mem.read_unaligned(address as usize, 0x0fff)
    }

    fn write_data<T: Size>(&mut self, address: u32, value: T) {
        self.rsp
            .mem
            .write_unaligned(address as usize, 0x0fff, value);
    }

    fn read_register(&self, index: usize) -> u32 {
        if index < 8 {
            self.rsp.read_register(index)
        } else {
            self.rdp.read_register(index - 8)
        }
    }

    fn write_register(&mut self, index: usize, value: u32) {
        let mask = WriteMask::unmasked(value);

        if index < 8 {
            self.rsp.write_register(index, mask);
        } else {
            self.rdp.write_register(index - 8, mask);
        }
    }

    fn break_(&mut self) {
        self.rsp.regs.status.set_halted(true);
        self.rsp.regs.status.set_broke(true);

        if self.rsp.regs.status.intbreak() {
            self.rsp.rcp_int.raise(RcpIntType::SP);
        }
    }
}
