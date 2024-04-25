use super::renderer;
use super::renderer::{Format, Renderer};
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use tracing::warn;

mod mode;
mod param;
mod rect;
mod sync;
mod target;
mod tmem;
mod triangle;

pub struct Context<'a> {
    pub renderer: &'a mut Renderer,
    pub rdram: &'a RwLock<Rdram>,
    pub gfx: &'a GfxContext,
}

pub struct Decoder {
    receiver: Receiver<u64>,
    running: Arc<AtomicBool>,
    pending_command: Option<u64>,
    params: VecDeque<u64>,
}

impl Decoder {
    pub fn new(receiver: Receiver<u64>, running: Arc<AtomicBool>) -> Self {
        Self {
            receiver,
            running,
            pending_command: None,
            params: VecDeque::new(),
        }
    }

    pub fn running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn step(&mut self, bus: Context) -> bool {
        let word = match self.pending_command.take() {
            Some(word) => word,
            None => match self.receiver.try_recv() {
                Ok(word) => word,
                Err(_) => {
                    self.halt();
                    return false;
                }
            },
        };

        let opcode = (word >> 56) & 0x3f;

        match opcode {
            0x00 => (), // NOP
            0x08 => triangle::triangle::<false, false, false>(self, bus, word),
            0x09 => triangle::triangle::<false, false, true>(self, bus, word),
            0x0a => triangle::triangle::<false, true, false>(self, bus, word),
            0x0b => triangle::triangle::<false, true, true>(self, bus, word),
            0x0c => triangle::triangle::<true, false, false>(self, bus, word),
            0x0d => triangle::triangle::<true, false, true>(self, bus, word),
            0x0e => triangle::triangle::<true, true, false>(self, bus, word),
            0x0f => triangle::triangle::<true, true, true>(self, bus, word),
            0x24 => rect::rectangle::<true, false>(self, bus, word),
            0x25 => rect::rectangle::<true, true>(self, bus, word),
            0x26 => sync::sync_load(self, bus, word),
            0x27 => sync::sync_pipe(self, bus, word),
            0x28 => sync::sync_tile(self, bus, word),
            0x29 => sync::sync_full(self, bus, word),
            0x2d => target::set_scissor(self, bus, word),
            0x2e => param::set_prim_depth(self, bus, word),
            0x2f => mode::set_other_modes(self, bus, word),
            0x30 => tmem::load_tlut(self, bus, word),
            0x32 => tmem::set_tile_size(self, bus, word),
            0x33 => tmem::load_block(self, bus, word),
            0x34 => tmem::load_tile(self, bus, word),
            0x35 => tmem::set_tile(self, bus, word),
            0x36 => rect::rectangle::<false, false>(self, bus, word),
            0x37 => target::set_fill_color(self, bus, word),
            0x38 => param::set_fog_color(self, bus, word),
            0x39 => param::set_blend_color(self, bus, word),
            0x3a => param::set_prim_color(self, bus, word),
            0x3b => param::set_env_color(self, bus, word),
            0x3c => mode::set_combine_mode(self, bus, word),
            0x3d => tmem::set_texture_image(self, bus, word),
            0x3f => target::set_color_image(self, bus, word),
            _ => warn!("TODO: RDP Command: {:#02X}", opcode),
        }

        // If SYNC_FULL was run, let the caller know
        opcode == 0x29
    }

    fn halt(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Format {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value {
            0 => Self::Rgba,
            1 => Self::Yuv,
            2 => Self::ClrIndex,
            3 => Self::IA,
            _ => Self::I,
        }
    }
}
