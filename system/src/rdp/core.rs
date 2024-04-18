use super::renderer;
use super::renderer::Renderer;
use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use std::collections::VecDeque;
use tracing::warn;

mod image;
mod mode;
mod param;
mod rect;
mod sync;
mod triangle;

pub struct Bus<'a> {
    pub renderer: &'a mut Renderer,
    pub rdram: &'a mut Rdram,
    pub gfx: &'a GfxContext,
}

pub struct Core {
    running: bool,
    commands: VecDeque<u64>,
}

impl Core {
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

    pub fn step(&mut self, bus: Bus) -> bool {
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
            0x27 => sync::sync_pipe(self, bus, word),
            0x29 => sync::sync_full(self, bus, word),
            0x2d => image::set_scissor(self, bus, word),
            0x2e => param::set_prim_depth(self, bus, word),
            0x2f => mode::set_other_modes(self, bus, word),
            0x36 => rect::rectangle::<false, false>(self, bus, word),
            0x37 => param::set_fill_color(self, bus, word),
            0x39 => param::set_blend_color(self, bus, word),
            0x3f => image::set_color_image(self, bus, word),
            _ => warn!("TODO: RDP Command: {:#02X}", opcode),
        }

        // If SYNC_FULL was run, let the caller know
        opcode == 0x29
    }
}
