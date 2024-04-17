pub use target::{ColorImage, ColorImageFormat};

use crate::gfx::GfxContext;
use crate::rdram::Rdram;
use display_list::{DisplayList, Vertex};
use target::Target;
use tracing::trace;

mod display_list;
mod target;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
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
                    bind_group_layouts: &[&scissor_bind_group_layout],
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
                            blend: Some(wgpu::BlendState::REPLACE),
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
            target: Target::new(gfx, &scissor_bind_group_layout),
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
            self.target.upload_buffers(gfx.queue());
            self.flush(gfx, rdram);
        }
    }

    pub fn set_fill_color(&mut self, packed_color: u32) {
        self.fill_color = packed_color;
        trace!("  Fill Color: {:08X}", self.fill_color);
    }

    pub fn draw_triangle(&mut self, edges: &[[f32; 2]; 3]) {
        self.display_list.push_triangle(edges, self.fill_color);
    }

    pub fn draw_rectangle(&mut self, rect: Rect) {
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

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RDP Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.target.scissor_bind_group(), &[]);

            let scissor = self.target.scissor();
            render_pass.set_viewport(0.0, 0.0, scissor.width(), scissor.height(), 0.0, 1.0);

            self.display_list.flush(&mut render_pass);
        }

        gfx.queue().submit(std::iter::once(encoder.finish()));

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
