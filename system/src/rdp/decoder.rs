use super::renderer;
use super::renderer::Renderer;
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::collections::VecDeque;
use tracing::warn;

mod mode;
mod param;
mod rect;
mod sync;
mod target;
mod tmem;
mod triangle;

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Format {
    Rgba = 0,
    Yuv = 1,
    ColorIndex = 2,
    IA = 3,
    I = 4,
}

impl Format {
    const fn into_bits(self) -> u32 {
        self as u32
    }

    const fn from_bits(value: u32) -> Self {
        match value & 3 {
            0 => Self::Rgba,
            1 => Self::Yuv,
            2 => Self::ColorIndex,
            3 => Self::IA,
            _ => Self::I,
        }
    }
}

pub struct Context<'a> {
    pub renderer: &'a mut Renderer,
    pub rdram: &'a mut Rdram,
    pub gfx: &'a GfxContext,
}

pub struct Decoder {
    running: bool,
    commands: VecDeque<u64>,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            running: false,
            commands: VecDeque::new(),
        }
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn restart(&mut self) {
        self.running = true;
    }

    pub fn write_command(&mut self, value: u64) {
        self.commands.push_back(value)
    }

    pub fn step(&mut self, bus: Context) -> bool {
        let Some(word) = self.commands.pop_front() else {
            self.running = false;
            return false;
        };

        let opcode = (word >> 56) & 0x3f;

        match opcode {
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
            0x32 => tmem::set_tile_size(self, bus, word),
            0x34 => tmem::load_tile(self, bus, word),
            0x35 => tmem::set_tile(self, bus, word),
            0x36 => rect::rectangle::<false, false>(self, bus, word),
            0x37 => param::set_fill_color(self, bus, word),
            0x39 => param::set_blend_color(self, bus, word),
            0x3d => tmem::set_texture_image(self, bus, word),
            0x3f => target::set_color_image(self, bus, word),
            _ => warn!("TODO: RDP Command: {:#02X}", opcode),
        }

        // If SYNC_FULL was run, let the caller know
        opcode == 0x29
    }
}
