pub use target::{ColorImage, ColorImageFormat};

use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use display_list::{DisplayList, Vertex};
use target::Target;
use tracing::trace;

mod display_list;
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
    display_list: DisplayList,
    render_pipeline: wgpu::RenderPipeline,
    fill_color: u32,
}

impl Renderer {
    pub fn new(gfx: &GfxContext) -> Self {
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
                    bind_group_layouts: &[],
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        Self {
            target: Target::new(),
            display_list: DisplayList::new(gfx.device()),
            render_pipeline,
            fill_color: 0,
        }
    }

    pub fn set_color_image(
        &mut self,
        gfx: &GfxContext,
        rdram: &mut Rdram,
        color_image: ColorImage,
    ) {
        self.target.set_color_image(color_image);

        if self.target.is_dirty() {
            self.flush(gfx, rdram);
        }
    }

    pub fn set_scissor(&mut self, gfx: &GfxContext, rdram: &mut Rdram, rect: Rect) {
        self.target.set_scissor(rect);

        if self.target.is_dirty() {
            self.flush(gfx, rdram);
        }
    }

    pub fn set_fill_color(&mut self, packed_color: u32) {
        self.fill_color = packed_color;
        trace!("  Fill Color: {:08X}", self.fill_color);
    }

    pub fn push_rectangle(&mut self, rect: Rect) {
        self.display_list.push_rectangle(rect, self.fill_color);
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

        self.display_list.upload(gfx.queue());

        // Render the scene
        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RDP Command Encoder"),
            });

        {
            let color_texture_view = output
                .color_texture
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
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            self.display_list.flush(&mut render_pass);
        }

        self.target.request_sync();
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
