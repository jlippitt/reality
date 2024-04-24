use super::{BlendMode, CombineMode, Rect, Tmem};
use bytemuck::{Pod, Zeroable};
use pod_enum::pod_enum;
use std::mem;
use std::ops::Range;
use tracing::trace;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FixedColor {
    Fog = 0,
    Blend = 1,
    Primitive = 2,
    Environment = 3,
}

#[pod_enum]
#[repr(u32)]
#[derive(Default, Eq)]
pub enum CycleType {
    OneCycle = 0,
    TwoCycle = 1,
    Copy = 2,
    Fill = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct Constants {
    combine_mode: CombineMode,
    blend_mode: BlendMode,
    fixed_colors: [[f32; 4]; 4],
    cycle_type: CycleType,
}

#[derive(Debug)]
enum Command {
    Triangles(Range<u32>),
    Rectangles(Range<u32>),
    SetTexture(Option<u128>),
    SetConstants(Range<u32>),
}

pub struct DisplayList {
    commands: Vec<Command>,
    constants: Constants,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    constant_data: Vec<u8>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    constant_buffer: wgpu::Buffer,
    constant_buffer_stride: u32,
    constant_bind_group_layout: wgpu::BindGroupLayout,
    constant_bind_group: wgpu::BindGroup,
    current_texture_handle: Option<u128>,
}

impl DisplayList {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Vertex Buffer"),
            size: 262144,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Index Buffer"),
            size: 131072,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let alignment = device.limits().min_storage_buffer_offset_alignment;
        let constant_buffer_stride =
            (mem::size_of::<Constants>() as u32).next_multiple_of(alignment);
        let binding_size = wgpu::BufferSize::new(constant_buffer_stride as u64);

        let constant_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("RDP Constant Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: binding_size,
                    },
                    count: None,
                }],
            });

        let constant_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Constant Buffer"),
            size: device.limits().max_uniform_buffer_binding_size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let constant_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RDP Constant Bind Group"),
            layout: &constant_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &constant_buffer,
                    offset: 0,
                    size: binding_size,
                }),
            }],
        });

        let mut display_list = Self {
            commands: vec![],
            vertices: vec![],
            indices: vec![],
            constants: Default::default(),
            constant_data: vec![],
            vertex_buffer,
            index_buffer,
            constant_buffer,
            constant_buffer_stride,
            constant_bind_group_layout,
            constant_bind_group,
            current_texture_handle: None,
        };

        display_list.reset();
        display_list
    }

    pub fn constant_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.constant_bind_group_layout
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn cycle_type(&self) -> CycleType {
        self.constants.cycle_type
    }

    pub fn set_combine_mode(&mut self, combine_mode: CombineMode) {
        let prev_value = self.constants.combine_mode;
        self.constants.combine_mode = combine_mode;
        trace!("  RGB Cycle 0: {}", self.constants.combine_mode.rgb[0]);
        trace!("  RGB Cycle 1: {}", self.constants.combine_mode.rgb[1]);
        trace!("  Alpha Cycle 0: {}", self.constants.combine_mode.alpha[0]);
        trace!("  Alpha Cycle 1: {}", self.constants.combine_mode.alpha[1]);

        if prev_value != self.constants.combine_mode {
            self.push_constants();
        }
    }

    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        let prev_value = self.constants.blend_mode;
        self.constants.blend_mode = blend_mode;
        trace!("  Blend Cycle 0: {}", self.constants.blend_mode.mode[0]);
        trace!("  Blend Cycle 1: {}", self.constants.blend_mode.mode[1]);

        if prev_value != self.constants.blend_mode {
            self.push_constants();
        }
    }

    pub fn set_fixed_color(&mut self, color: FixedColor, value: u32) {
        let fixed_colors = &mut self.constants.fixed_colors;
        let index = color as usize;

        let normalised_value = [
            (value >> 24) as f32 / 255.0,
            ((value >> 16) & 0xff) as f32 / 255.0,
            ((value >> 8) & 0xff) as f32 / 255.0,
            (value & 0xff) as f32 / 255.0,
        ];

        let prev_value = fixed_colors[index];
        fixed_colors[index] = normalised_value;
        trace!("  {:?} Color: {:?}", color, fixed_colors[index]);

        if prev_value != fixed_colors[index] {
            self.push_constants();
        }
    }

    pub fn set_cycle_type(&mut self, cycle_type: CycleType) {
        let prev_value = self.constants.cycle_type;
        self.constants.cycle_type = cycle_type;
        trace!("  Cycle Type: {:?}", self.constants.cycle_type);

        if prev_value != self.constants.cycle_type {
            self.push_constants();
        }
    }

    fn push_constants(&mut self) {
        let size = mem::size_of::<Constants>();
        let constants = bytemuck::bytes_of(&self.constants);

        match self.commands.last_mut() {
            Some(Command::SetConstants(Range { start, .. })) => {
                self.constant_data[*start as usize..(*start as usize + size)]
                    .copy_from_slice(constants);
            }
            _ => {
                let start = self.constant_data.len() as u32;

                self.constant_data.extend_from_slice(constants);

                // Pad to uniform buffer alignment required by hardware drivers
                self.constant_data.resize(
                    self.constant_data.len() + self.constant_buffer_stride as usize - size,
                    0,
                );

                self.commands.push(Command::SetConstants(
                    start..(start + self.constant_buffer_stride),
                ));
            }
        }
    }

    pub fn push_triangle(
        &mut self,
        edges: [[f32; 2]; 3],
        colors: [[f32; 4]; 3],
        texture: Option<(u128, [[f32; 3]; 3])>,
        z_values: [f32; 3],
    ) -> bool {
        let (handle, tex_coords) = if let Some((handle, tex_coords)) = texture {
            (Some(handle), tex_coords)
        } else {
            (None, [[0.0; 3]; 3])
        };

        if handle != self.current_texture_handle {
            self.commands.push(Command::SetTexture(handle));
            self.current_texture_handle = handle;
        }

        let vertices = [
            Vertex {
                position: [edges[0][0], edges[0][1], z_values[0]],
                color: colors[0],
                tex_coords: tex_coords[0],
            },
            Vertex {
                position: [edges[1][0], edges[1][1], z_values[1]],
                color: colors[1],
                tex_coords: tex_coords[1],
            },
            Vertex {
                position: [edges[2][0], edges[2][1], z_values[2]],
                color: colors[2],
                tex_coords: tex_coords[2],
            },
        ];

        self.vertices.extend_from_slice(&vertices);

        let end = self.vertices.len().try_into().unwrap();

        match self.commands.last_mut() {
            Some(Command::Triangles(existing_range)) => {
                *existing_range = existing_range.start..end;
            }
            _ => {
                self.commands.push(Command::Triangles((end - 3)..end));
            }
        }

        let vertex_size = mem::size_of::<Vertex>();

        (self.vertices.len() * vertex_size) >= (self.vertex_buffer.size() as usize - vertex_size)
    }

    pub fn push_rectangle(
        &mut self,
        rect: Rect,
        fill_color: [f32; 4],
        texture: Option<(u128, Rect, bool)>,
        z_value: f32,
    ) -> bool {
        let (handle, tex_coords) = if let Some((handle, tex_rect, flip)) = texture {
            (
                Some(handle),
                if flip {
                    [
                        [tex_rect.left, tex_rect.top, 0.0],
                        [tex_rect.right, tex_rect.top, 0.0],
                        [tex_rect.left, tex_rect.bottom, 0.0],
                        [tex_rect.right, tex_rect.bottom, 0.0],
                    ]
                } else {
                    [
                        [tex_rect.left, tex_rect.top, 0.0],
                        [tex_rect.left, tex_rect.bottom, 0.0],
                        [tex_rect.right, tex_rect.top, 0.0],
                        [tex_rect.right, tex_rect.bottom, 0.0],
                    ]
                },
            )
        } else {
            (None, [[0.0; 3]; 4])
        };

        if handle != self.current_texture_handle {
            self.commands.push(Command::SetTexture(handle));
            self.current_texture_handle = handle;
        }

        let vertices = [
            Vertex {
                position: [rect.left, rect.top, z_value],
                color: fill_color,
                tex_coords: tex_coords[0],
            },
            Vertex {
                position: [rect.left, rect.bottom, z_value],
                color: fill_color,
                tex_coords: tex_coords[1],
            },
            Vertex {
                position: [rect.right, rect.top, z_value],
                color: fill_color,
                tex_coords: tex_coords[2],
            },
            Vertex {
                position: [rect.right, rect.bottom, z_value],
                color: fill_color,
                tex_coords: tex_coords[3],
            },
        ];

        let base_vertex = self.vertices.len().try_into().unwrap();

        self.vertices.extend_from_slice(&vertices);

        self.indices.extend_from_slice(&[
            base_vertex,
            base_vertex + 1,
            base_vertex + 2,
            base_vertex + 2,
            base_vertex + 1,
            base_vertex + 3,
        ]);

        let end = self.indices.len().try_into().unwrap();

        match self.commands.last_mut() {
            Some(Command::Rectangles(existing_range)) => {
                *existing_range = existing_range.start..end;
            }
            _ => {
                self.commands.push(Command::Rectangles((end - 6)..end));
            }
        }

        let vertex_size = mem::size_of::<Vertex>();
        let index_size = mem::size_of::<u32>();

        (self.vertices.len() * vertex_size) >= (self.vertex_buffer.size() as usize - vertex_size)
            || (self.indices.len() * index_size) >= (self.index_buffer.size() as usize - index_size)
    }

    pub fn reset(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.constant_data.clear();
        self.current_texture_handle = None;
        self.push_constants();
    }

    pub fn upload_buffers(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        queue.write_buffer(
            &self.constant_buffer,
            0,
            bytemuck::cast_slice(&self.constant_data),
        );
    }

    pub fn flush<'a>(&'a mut self, tmem: &'a Tmem, render_pass: &mut wgpu::RenderPass<'a>) {
        trace!("  Display List: {:?}", self.commands);
        trace!("  Vertices: {:?}", self.vertices);
        trace!("  Indices: {:?}", self.indices);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for command in self.commands.drain(..) {
            match command {
                Command::Triangles(range) => render_pass.draw(range.clone(), 0..1),
                Command::Rectangles(range) => render_pass.draw_indexed(range.clone(), 0, 0..1),
                Command::SetTexture(handle) => {
                    render_pass.set_bind_group(2, tmem.bind_group(handle), &[])
                }
                Command::SetConstants(range) => {
                    render_pass.set_bind_group(3, &self.constant_bind_group, &[range.start])
                }
            }
        }
    }
}
