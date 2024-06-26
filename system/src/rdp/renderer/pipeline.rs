use super::display_list::Vertex;
use tracing::trace;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PipelineSpec {
    pub z_compare_en: bool,
    pub z_update_en: bool,
}

pub struct Pipeline {
    layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    current: wgpu::RenderPipeline,
    spec: PipelineSpec,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        layout: wgpu::PipelineLayout,
        shader: wgpu::ShaderModule,
    ) -> Self {
        let spec = PipelineSpec::default();
        let current = create_from_spec(device, &layout, &shader, &spec);

        Self {
            layout,
            shader,
            current,
            spec,
        }
    }

    pub fn current(&self) -> &wgpu::RenderPipeline {
        &self.current
    }

    pub fn spec(&self) -> &PipelineSpec {
        &self.spec
    }

    pub fn update(&mut self, device: &wgpu::Device, spec: PipelineSpec) {
        if spec == self.spec {
            return;
        }

        self.current = create_from_spec(device, &self.layout, &self.shader, &spec);
        self.spec = spec;
    }
}

fn create_from_spec(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    spec: &PipelineSpec,
) -> wgpu::RenderPipeline {
    trace!("  Pipeline: {:?}", spec);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("RDP Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
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
            format: wgpu::TextureFormat::Depth16Unorm,
            depth_write_enabled: spec.z_update_en,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        // depth_stencil: spec.z_compare_en.then_some(wgpu::DepthStencilState {
        //     format: wgpu::TextureFormat::Depth16Unorm,
        //     depth_write_enabled: spec.z_update_en,
        //     depth_compare: wgpu::CompareFunction::LessEqual,
        //     stencil: wgpu::StencilState::default(),
        //     bias: wgpu::DepthBiasState::default(),
        // }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
