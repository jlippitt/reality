pub use target::ColorImageFormat;

use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use target::Target;
use tracing::trace;

mod target;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rect {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

pub struct Renderer {
    target: Target,
    fill_color: u32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            target: Target::new(),
            fill_color: 0,
        }
    }

    pub fn set_color_image(&mut self, dram_addr: u32, width: u32, format: ColorImageFormat) {
        self.target.set_color_image(dram_addr, width, format);
    }

    pub fn set_scissor(&mut self, rect: Rect) {
        self.target.set_scissor(rect);
    }

    pub fn set_fill_color(&mut self, packed_color: u32) {
        self.fill_color = packed_color;
        trace!("  Fill Color: {:08X}", self.fill_color);
    }

    pub fn sync(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        self.flush(gfx, rdram);
        self.target.sync(gfx, rdram);
    }

    pub fn flush(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        self.target.update(gfx, rdram);

        let Some(_output) = self.target.output() else {
            return;
        };

        // TODO: Draw the scene
    }
}

impl Rect {
    fn width(&self) -> u32 {
        self.right - self.left
    }

    fn height(&self) -> u32 {
        self.bottom - self.top
    }
}
