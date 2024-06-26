pub use blender::{BlendModeRaw, BlendModeRawParams};
pub use combiner::{CombineModeRaw, CombineModeRawParams};
pub use display_list::{CycleType, FixedColor};
pub use target::ColorImage;
pub use tmem::{TextureImage, TileAddressMode, TileDescriptor};

use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use blender::BlendMode;
use combiner::CombineMode;
use display_list::DisplayList;
use pipeline::{Pipeline, PipelineSpec};
use target::Target;
use tmem::Tmem;
use tracing::trace;

mod blender;
mod combiner;
mod display_list;
mod pipeline;
mod target;
mod tmem;

const DEFAULT_W_VALUE: f32 = 1024.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Format {
    #[default]
    Rgba = 0,
    Yuv = 1,
    ClrIndex = 2,
    IA = 3,
    I = 4,
}

pub type TextureFormat = (Format, u32);

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ZSource {
    #[default]
    PerPixel = 0,
    Primitive = 1,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum SampleType {
    #[default]
    Point = 0,
    Bilinear = 1,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct OtherModes {
    pub cycle_type: CycleType,
    pub sample_type: SampleType,
    pub perspective_enable: bool,
    pub z_compare_en: bool,
    pub z_update_en: bool,
    pub z_source: ZSource,
    pub blend_mode: BlendModeRaw,
}

pub struct Renderer {
    target: Target,
    tmem: Tmem,
    display_list: DisplayList,
    pipeline: Pipeline,
    sample_type: SampleType,
    perspective_enable: bool,
    z_source: ZSource,
    prim_depth: f32,
    blend_color: [f32; 4],
}

impl Renderer {
    pub fn new(gfx: &GfxContext) -> Self {
        let target = Target::new(gfx);
        let tmem = Tmem::new(gfx);
        let display_list = DisplayList::new(gfx.device());

        let shader = gfx
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RDP Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("renderer.wgsl").into()),
            });

        let render_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("RDP Render Pipeline Layout"),
                    bind_group_layouts: &[
                        target.image_size_bind_group_layout(),
                        target.fill_color_bind_group_layout(),
                        tmem.bind_group_layout(),
                        display_list.constant_bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline = Pipeline::new(gfx.device(), render_pipeline_layout, shader);

        Self {
            target,
            tmem,
            display_list,
            pipeline,
            sample_type: SampleType::default(),
            perspective_enable: false,
            z_source: ZSource::default(),
            prim_depth: 0.0,
            blend_color: [0.0; 4],
        }
    }

    pub fn set_color_image(
        &mut self,
        gfx: &GfxContext,
        rdram: &mut Rdram,
        color_image: ColorImage,
    ) {
        if color_image != *self.target.color_image() {
            self.flush(gfx, rdram);
        }

        self.target.set_color_image(color_image);
    }

    pub fn set_scissor(&mut self, gfx: &GfxContext, rdram: &mut Rdram, scissor: Rect) {
        if scissor != *self.target.scissor() {
            self.flush(gfx, rdram);
        }

        self.target.set_scissor(scissor);
    }

    pub fn set_fill_color(&mut self, gfx: &GfxContext, rdram: &mut Rdram, value: u32) {
        if value != self.target.fill_color() {
            self.flush(gfx, rdram);
        }

        self.target.set_fill_color(gfx.queue(), value);
    }

    pub fn set_combine_mode(&mut self, combine_mode: CombineModeRaw) {
        let combine_mode = CombineMode::from_raw(combine_mode);
        self.display_list.set_combine_mode(combine_mode);
    }

    pub fn set_other_modes(&mut self, gfx: &GfxContext, rdram: &mut Rdram, mode: OtherModes) {
        let pipeline_spec = PipelineSpec {
            z_compare_en: mode.z_compare_en,
            z_update_en: mode.z_update_en,
        };

        if pipeline_spec != *self.pipeline.spec() {
            self.flush(gfx, rdram);
            self.pipeline.update(gfx.device(), pipeline_spec);
        }

        self.display_list.set_cycle_type(mode.cycle_type);

        let blend_mode = BlendMode::from_raw(mode.blend_mode);
        self.display_list.set_blend_mode(blend_mode);

        self.sample_type = mode.sample_type;
        self.perspective_enable = mode.perspective_enable;
        self.z_source = mode.z_source;
        trace!("  Sample Type: {:?}", self.sample_type);
        trace!("  Perspective Enable: {:?}", self.perspective_enable);
        trace!("  Z Source: {:?}", self.z_source);
    }

    pub fn set_texture_image(&mut self, texture_image: TextureImage) {
        self.tmem.set_texture_image(texture_image);
    }

    pub fn set_tile(&mut self, index: usize, tile: TileDescriptor, hash_value: u64) {
        self.tmem.set_tile(index, tile, hash_value);
    }

    pub fn set_tile_size(&mut self, index: usize, rect: Rect, hash_value: u64) {
        self.tmem.set_tile_size(index, rect, hash_value);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load_tile(
        &mut self,
        gfx: &GfxContext,
        rdram: &mut Rdram,
        index: usize,
        x_offset: usize,
        x_size: usize,
        y_offset: usize,
        y_size: usize,
    ) {
        self.flush(gfx, rdram);
        self.tmem
            .load_tile(rdram, index, x_offset, x_size, y_offset, y_size);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load_tlut(
        &mut self,
        gfx: &GfxContext,
        rdram: &mut Rdram,
        index: usize,
        x_offset: usize,
        x_size: usize,
        y_offset: usize,
        y_size: usize,
    ) {
        self.flush(gfx, rdram);
        self.tmem
            .load_tlut(rdram, index, x_offset, x_size, y_offset, y_size);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load_block(
        &mut self,
        gfx: &GfxContext,
        rdram: &mut Rdram,
        index: usize,
        x_offset: usize,
        x_size: usize,
        y_offset: usize,
        y_delta: usize,
    ) {
        self.flush(gfx, rdram);
        self.tmem
            .load_block(rdram, index, x_offset, x_size, y_offset, y_delta);
    }

    pub fn set_fixed_color(&mut self, color: FixedColor, value: u32) {
        self.display_list.set_fixed_color(color, value);
    }

    pub fn set_prim_depth(&mut self, prim_depth: f32) {
        self.prim_depth = prim_depth;
        trace!("  Prim Depth: {}", self.prim_depth);
    }

    pub fn draw_triangle(
        &mut self,
        gfx: &GfxContext,
        rdram: &mut Rdram,
        edges: [[f32; 2]; 3],
        colors: [[f32; 4]; 3],
        texture: Option<(usize, [[f32; 3]; 3])>,
        z_values: [f32; 3],
    ) {
        let texture = texture.and_then(|(tile_id, mut coords)| {
            // Make texture coordinates relative to tile origin
            let tile_size = self.tmem.tile_size(tile_id);

            for vertex in &mut coords {
                vertex[0] -= tile_size.left;
                vertex[1] -= tile_size.top;
            }

            if !self.perspective_enable {
                for vertex in &mut coords {
                    vertex[2] = DEFAULT_W_VALUE;
                }
            }

            self.tmem
                .get_texture_handle(gfx, tile_id)
                .map(|handle| (handle, coords))
        });

        let z_values = if self.z_source == ZSource::Primitive {
            [self.prim_depth; 3]
        } else {
            z_values
        };

        if self
            .display_list
            .push_triangle(edges, colors, texture, z_values)
        {
            self.flush(gfx, rdram);
        }
    }

    pub fn draw_rectangle(
        &mut self,
        gfx: &GfxContext,
        rdram: &mut Rdram,
        mut rect: Rect,
        texture: Option<(usize, Rect, bool)>,
    ) {
        // TODO: Proper blending
        let color = self.blend_color;

        if self.display_list.cycle_type() == CycleType::Fill
            || self.display_list.cycle_type() == CycleType::Copy
        {
            rect.right += 1.0;
            rect.bottom += 1.0;
        }

        let texture = texture.and_then(|(tile_id, mut tex_rect, flip)| {
            self.tmem.get_texture_handle(gfx, tile_id).map(|handle| {
                // Make texture coordinates relative to tile origin
                let tile_size = self.tmem.tile_size(tile_id);
                tex_rect.left -= tile_size.left;
                tex_rect.right -= tile_size.left;
                tex_rect.top -= tile_size.top;
                tex_rect.bottom -= tile_size.top;

                if self.display_list.cycle_type() == CycleType::Copy {
                    // In copy mode, every 4 S steps is one pixel
                    tex_rect.right = (tex_rect.left * 3.0 + tex_rect.right) / 4.0;
                    tex_rect.right += (tex_rect.width() + 1.0) / rect.width();
                    tex_rect.bottom += (tex_rect.height() + 1.0) / rect.height();
                    trace!("  = {:?}", tex_rect);
                    // } else if self.sample_type == SampleType::Bilinear {
                    //     tex_rect.left += 0.5;
                    //     tex_rect.right += 0.5;
                    //     tex_rect.top += 0.5;
                    //     tex_rect.bottom += 0.5;
                }

                (handle, tex_rect, flip)
            })
        });

        let z_value = if self.z_source == ZSource::Primitive {
            self.prim_depth
        } else {
            0.0
        };

        if self
            .display_list
            .push_rectangle(rect, color, texture, z_value)
        {
            self.flush(gfx, rdram);
        }
    }

    pub fn sync(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        self.flush(gfx, rdram);
        self.target.sync(gfx, rdram);
    }

    pub fn flush(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        if self.display_list.is_empty() {
            return;
        }

        if !self.target.update(gfx, rdram) {
            return;
        }

        let Some(output) = self.target.output() else {
            return;
        };

        self.display_list.upload_buffers(gfx.queue());

        // Render the scene
        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RDP Render Command Encoder"),
            });

        {
            let color_texture_view = output
                .color_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let depth_texture_view = output
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RDP Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        // TODO: This should be loaded from RDRAM. For now,
                        // clear it to the max depth value.
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                // depth_stencil_attachment: self.pipeline.spec().z_compare_en.then_some(
                //     wgpu::RenderPassDepthStencilAttachment {
                //         view: &depth_texture_view,
                //         depth_ops: Some(wgpu::Operations {
                //             // TODO: This should be loaded from RDRAM. For now,
                //             // clear it to the max depth value.
                //             load: wgpu::LoadOp::Load,
                //             store: wgpu::StoreOp::Store,
                //         }),
                //         stencil_ops: None,
                //     },
                // ),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(self.pipeline.current());
            render_pass.set_bind_group(0, self.target.image_size_bind_group(), &[]);
            render_pass.set_bind_group(1, self.target.fill_color_bind_group(), &[]);
            render_pass.set_bind_group(2, self.tmem.bind_group(None), &[]);

            render_pass.set_viewport(
                0.0,
                0.0,
                output.color_image.width as f32,
                output.color_texture.height() as f32,
                0.0,
                1.0,
            );

            let scissor = self.target.scissor();

            render_pass.set_scissor_rect(
                scissor.left as u32,
                scissor.top as u32,
                scissor.width() as u32,
                scissor.height() as u32,
            );

            self.display_list.flush(&self.tmem, &mut render_pass);
        }

        gfx.queue().submit(std::iter::once(encoder.finish()));

        self.display_list.reset();
        self.target.request_sync();
    }
}

impl Rect {
    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn height(&self) -> f32 {
        self.bottom - self.top
    }
}
