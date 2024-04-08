use super::memory::{Memory, Size, WriteMask};
use super::rdram::Rdram;
use core::Core;
use regs::{DmaLength, DmaRamAddr, DmaSpAddr, Regs, Status};
use std::mem;
use tracing::{debug, trace, warn};

mod core;
mod regs;

const MEM_SIZE: usize = 8192;

#[derive(Debug)]
struct Dma {
    sp_addr: DmaSpAddr,
    ram_addr: DmaRamAddr,
    len: DmaLength,
    write: bool,
}

struct Bus {
    mem: Memory<u128>,
    regs: Regs,
    dma_active: Option<Dma>,
    dma_pending: Option<Dma>,
}

pub struct Rsp {
    core: Core,
    bus: Bus,
}

impl Rsp {
    pub fn new(ipl3_data: Option<&[u8]>) -> Self {
        let mem = if let Some(ipl3_data) = ipl3_data {
            let mut vec = Vec::from(ipl3_data);
            vec.resize(MEM_SIZE, 0);
            Memory::from_bytes(&vec)
        } else {
            Memory::with_byte_len(MEM_SIZE)
        };

        Self {
            core: Core::new(),
            bus: Bus {
                mem,
                dma_active: None,
                dma_pending: None,
                regs: Regs::default(),
            },
        }
    }

    pub fn step_core(&mut self) {
        if self.bus.regs.status.halted() {
            return;
        }

        self.core.step(&mut self.bus);
        let status = &mut self.bus.regs.status;
        status.set_halted(status.halted() | status.sstep());
    }

    pub fn step_dma(&mut self, rdram: &mut Rdram) {
        let Some(dma_active) = &mut self.bus.dma_active else {
            return;
        };

        let bank_offset = (dma_active.sp_addr.mem_bank() as usize) << 12;
        let mem_addr = dma_active.sp_addr.mem_addr() as usize & 0x0ff8;
        let dram_addr = dma_active.ram_addr.dram_addr() as usize & 0x00ff_fff8;
        let block_len = dma_active.len.len().min(128) as usize;

        if dma_active.len.skip() != 0 || dma_active.len.count() != 0 {
            todo!("RSP DMA Skip/Count");
        }

        let mut buf = [0u8; 128];
        let data = &mut buf[0..block_len];

        if dma_active.write {
            let mut byte_addr = mem_addr;

            for byte in data.iter_mut() {
                *byte = self.bus.mem[bank_offset + byte_addr];
                byte_addr = (byte_addr + 1) & 0xff8;
            }

            rdram.write_block(dram_addr, data);

            debug!(
                "RSP DMA: {} bytes written from {:08X} to {:08X}",
                block_len,
                bank_offset | mem_addr,
                dram_addr,
            );
        } else {
            rdram.read_block(dram_addr, data);

            let mut byte_addr = mem_addr;

            for byte in data.iter() {
                self.bus.mem[bank_offset + byte_addr] = *byte;
                byte_addr = (byte_addr + 1) & 0xff8;
            }

            debug!(
                "RSP DMA: {} bytes read from {:08X} to {:08X}",
                block_len,
                dram_addr,
                bank_offset | mem_addr,
            );
        }

        let bytes_remaining = dma_active.len.len() as usize - block_len;

        if bytes_remaining == 0 {
            self.bus.dma_active = self.bus.dma_pending.take();
            trace!("RSP DMA Active: {:08X?}", self.bus.dma_active);

            if self.bus.dma_active.is_some() {
                trace!("RSP DMA Pending: {:08X?}", self.bus.dma_active);
            }
        } else {
            dma_active
                .ram_addr
                .set_dram_addr((dram_addr + block_len) as u32);

            dma_active
                .sp_addr
                .set_mem_addr((mem_addr + block_len) as u32);

            dma_active.len.set_len(bytes_remaining as u32);
        }
    }

    pub fn read<T: Size>(&self, address: u32) -> T {
        if (address as usize) < MEM_SIZE {
            return self.bus.mem.read(address as usize);
        }

        T::from_u32(if (address & 0x0004_0000) == 0x0004_0000 {
            self.bus.read_register(address)
        } else if address == 0x0008_0000 {
            self.core.pc()
        } else {
            panic!("Read from unmapped RSP address: {:08X}", address);
        })
    }

    pub fn write<T: Size>(&mut self, address: u32, value: T) {
        if (address as usize) < MEM_SIZE {
            return self.bus.mem.write(address as usize, value);
        }

        let mask = WriteMask::new(address, value);

        if (address & 0x0004_0000) == 0x0004_0000 {
            self.bus.write_register(address, mask);
        } else if address == 0x0008_0000 {
            let mut pc = self.core.pc();
            mask.write(&mut pc);
            self.core.set_pc(pc);
            debug!("RSP PC: {:08X}", pc);
        } else {
            panic!("Write to unmapped RSP address: {:08X}", address);
        }
    }
}

impl Bus {
    fn read_register(&self, address: u32) -> u32 {
        match (address & 0xffff) >> 2 {
            4 => self
                .regs
                .status
                .with_dma_busy(self.dma_active.is_some())
                .with_dma_full(self.dma_pending.is_some())
                .into(),
            6 => self.dma_active.is_some() as u32,
            _ => todo!("RSP Register Read: {:08X}", address),
        }
    }

    fn write_register(&mut self, address: u32, mask: WriteMask) {
        match (address & 0xffff) >> 2 {
            0 => mask.write_reg_hex("SP_DMA_SPADDR", &mut self.regs.dma_sp_addr),
            1 => mask.write_reg_hex("SP_DMA_RAMADDR", &mut self.regs.dma_ram_addr),
            2 => self.enqueue_dma(mask.raw(), false),
            3 => self.enqueue_dma(mask.raw(), true),
            4 => {
                let status = &mut self.regs.status;
                let raw = mask.raw();

                if (raw & 0x0000_0002) != 0 {
                    status.set_broke(false);
                }

                if (raw & 0x0000_0008) != 0 {
                    warn!("TODO: Acknowledge RSP Interrupt");
                }

                if (raw & 0x0000_0010) != 0 {
                    todo!("Trigger RSP interrupt");
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
            _ => todo!("RSP Register Write: {:08X} <= {:08X}", address, mask.raw()),
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
            write,
        };

        if self.dma_active.is_none() {
            self.dma_active = Some(dma);
            trace!("RSP DMA Active: {:08X?}", self.dma_active);
        } else if self.dma_pending.is_none() {
            self.dma_pending = Some(dma);
            trace!("RSP DMA Pending: {:08X?}", self.dma_pending);
        } else {
            panic!("RSP DMA queue full");
        }
    }
}

impl core::Bus for Bus {
    fn read_opcode(&self, address: u32) -> u32 {
        let address = address as usize;
        debug_assert!(address < (MEM_SIZE / 2));
        debug_assert!((address & 3) == 0);
        self.mem.read(0x1000 | address)
    }

    fn read_data<T: Size>(&self, address: u32) -> T {
        let address = address as usize;
        debug_assert!(address < (MEM_SIZE / 2));
        debug_assert!((address & (mem::size_of::<T>() - 1)) == 0);
        self.mem.read(address)
    }

    fn write_data<T: Size>(&mut self, address: u32, value: T) {
        let address = address as usize;
        debug_assert!(address < (MEM_SIZE / 2));
        debug_assert!((address & (mem::size_of::<T>() - 1)) == 0);
        self.mem.write(address, value)
    }
}
