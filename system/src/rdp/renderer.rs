pub use target::{ColorImage, ColorImageFormat};
pub use tmem::{TextureFormat, TextureImage, TileDescriptor};

use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use display_list::{DisplayList, Vertex};
use target::Target;
use tmem::Tmem;
use tracing::trace;

mod display_list;
mod target;
mod tmem;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum CycleType {
    #[default]
    OneCycle = 0,
    TwoCycle = 1,
    Copy = 2,
    Fill = 3,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ZSource {
    #[default]
    PerPixel = 0,
    Primitive = 1,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ZBufferConfig {
    pub enable: bool,
    pub write_enable: bool,
    pub source: ZSource,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mode {
    pub cycle_type: CycleType,
    pub z_buffer: ZBufferConfig,
}

pub struct Renderer {
    target: Target,
    tmem: Tmem,
    display_list: DisplayList,
    render_pipeline: wgpu::RenderPipeline,
    mode: Mode,
    blend_color: [f32; 4],
    fill_color: u32,
    prim_depth: f32,
}

impl Renderer {
    pub fn new(gfx: &GfxContext) -> Self {
        let tmem = Tmem::new(gfx);

        let shader = gfx
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RDP Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("renderer.wgsl").into()),
            });

        let scissor_bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RDP Scissor Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let render_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("RDP Render Pipeline Layout"),
                    bind_group_layouts: &[&scissor_bind_group_layout, tmem.bind_group_layout()],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            gfx.device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("RDP Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        Self {
            target: Target::new(gfx, &scissor_bind_group_layout),
            tmem,
            display_list: DisplayList::new(gfx.device()),
            render_pipeline,
            mode: Mode::default(),
            blend_color: [0.0; 4],
            fill_color: 0,
            prim_depth: 0.0,
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

        if self.target.is_dirty() {
            self.target.upload_buffers(gfx.queue());
        }
    }

    pub fn set_mode(&mut self, gfx: &GfxContext, rdram: &mut Rdram, mode: Mode) {
        if mode != self.mode {
            self.flush(gfx, rdram);
        }

        self.mode = mode;
        trace!("  Mode: {:?}", self.mode);
    }

    pub fn set_texture_image(&mut self, texture_image: TextureImage) {
        self.tmem.set_texture_image(texture_image);
    }

    pub fn set_tile(&mut self, index: usize, tile: TileDescriptor, hash_value: u64) {
        self.tmem.set_tile(index, tile, hash_value);
    }

    pub fn load_tile(&mut self, rdram: &Rdram, index: usize, rect: Rect, hash_value: u64) {
        self.tmem.load_tile(rdram, index, rect, hash_value);
    }

    pub fn blend_color(&self) -> [f32; 4] {
        self.blend_color
    }

    pub fn set_blend_color(&mut self, color: u32) {
        self.blend_color = decode_color(color);
        trace!("  Blend Color: {:?}", self.blend_color);
    }

    pub fn set_fill_color(&mut self, packed_color: u32) {
        self.fill_color = packed_color;
        trace!("  Fill Color: {:08X}", self.fill_color);
    }

    pub fn set_prim_depth(&mut self, prim_depth: f32) {
        self.prim_depth = prim_depth;
        trace!("  Prim Depth: {}", self.prim_depth);
    }

    pub fn draw_triangle(
        &mut self,
        _gfx: &GfxContext,
        edges: [[f32; 2]; 3],
        colors: [[f32; 4]; 3],
        z_values: [f32; 3],
    ) {
        let colors = if self.mode.cycle_type == CycleType::Fill {
            [self.fill_color(); 3]
        } else {
            colors
        };

        let z_values = if self.mode.z_buffer.source == ZSource::Primitive {
            [self.prim_depth; 3]
        } else {
            z_values
        };

        self.display_list.push_triangle(edges, colors, z_values);
    }

    pub fn draw_rectangle(&mut self, gfx: &GfxContext, rect: Rect, texture: Option<(usize, Rect)>) {
        let (color, texture) = if self.mode.cycle_type == CycleType::Fill {
            (self.fill_color(), None)
        } else {
            (
                // TODO: Proper blending
                self.blend_color,
                texture.map(|(tile_id, rect)| {
                    let handle = self.tmem.get_texture_handle(gfx, tile_id);
                    (handle, rect)
                }),
            )
        };

        let z_value = if self.mode.z_buffer.source == ZSource::Primitive {
            self.prim_depth
        } else {
            0.0
        };

        self.display_list
            .push_rectangle(rect, color, texture, z_value);
    }

    pub fn sync(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        self.flush(gfx, rdram);
        self.target.sync(gfx, rdram);
    }

    pub fn flush(&mut self, gfx: &GfxContext, rdram: &mut Rdram) {
        if self.display_list.is_empty() {
            return;
        }

        self.target.update(gfx, rdram);

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
                // TODO: Only add the stencil if 'self.mode.z_buffer.enable' is set
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        // TODO: This should be loaded from RDRAM. For now,
                        // clear it to the max depth value.
                        load: wgpu::LoadOp::Clear(1.0),
                        store: if self.mode.z_buffer.write_enable {
                            wgpu::StoreOp::Store
                        } else {
                            wgpu::StoreOp::Discard
                        },
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.target.scissor_bind_group(), &[]);
            render_pass.set_bind_group(1, self.tmem.bind_group(None), &[]);

            let scissor = self.target.scissor();
            render_pass.set_viewport(0.0, 0.0, scissor.width(), scissor.height(), 0.0, 1.0);

            self.display_list.flush(&self.tmem, &mut render_pass);
        }

        gfx.queue().submit(std::iter::once(encoder.finish()));

        self.target.request_sync();
    }

    fn fill_color(&self) -> [f32; 4] {
        match self.target.color_image().format {
            ColorImageFormat::ClrIndex8 => todo!("Index8 format"),
            ColorImageFormat::Rgba16 => [
                // This isn't correct, but it'll do for now
                (((self.fill_color >> 11) & 0x1f) << 3) as f32,
                (((self.fill_color >> 6) & 0x1f) << 3) as f32,
                (((self.fill_color >> 1) & 0x1f) << 3) as f32,
                ((self.fill_color & 0x01) * 255) as f32,
            ],
            ColorImageFormat::Rgba32 => decode_color(self.fill_color),
        }
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

fn decode_color(color: u32) -> [f32; 4] {
    [
        (color >> 24) as f32,
        ((color >> 16) & 0xff) as f32,
        ((color >> 8) & 0xff) as f32,
        (color & 0xff) as f32,
    ]
}
