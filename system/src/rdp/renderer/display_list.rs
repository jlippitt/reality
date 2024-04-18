use super::Rect;
use bytemuck::{Pod, Zeroable};
use std::ops::Range;
use tracing::trace;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug)]
enum Command {
    Triangles(Range<u32>),
    Rectangles(Range<u32>),
}

pub struct DisplayList {
    commands: Vec<Command>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
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
            label: Some("Index Buffer"),
            size: 131072,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            commands: vec![],
            vertices: vec![],
            indices: vec![],
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn push_triangle(&mut self, edges: [[f32; 2]; 3], colors: [[f32; 4]; 3]) {
        let vertices = [
            Vertex {
                position: edges[0],
                color: colors[0],
            },
            Vertex {
                position: edges[1],
                color: colors[1],
            },
            Vertex {
                position: edges[2],
                color: colors[2],
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
    }

    pub fn push_rectangle(&mut self, rect: Rect, fill_color: [f32; 4]) {
        let vertices = [
            Vertex {
                position: [rect.left, rect.top],
                color: fill_color,
            },
            Vertex {
                position: [rect.left, rect.bottom],
                color: fill_color,
            },
            Vertex {
                position: [rect.right, rect.top],
                color: fill_color,
            },
            Vertex {
                position: [rect.right, rect.bottom],
                color: fill_color,
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
    }

    pub fn upload_buffers(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
    }

    pub fn flush<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        trace!("  Display List: {:?}", self.commands);
        trace!("  Vertices: {:?}", self.vertices);
        trace!("  Indices: {:?}", self.indices);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for command in self.commands.drain(..) {
            match command {
                Command::Triangles(range) => render_pass.draw(range.clone(), 0..1),
                Command::Rectangles(range) => render_pass.draw_indexed(range.clone(), 0, 0..1),
            }
        }

        self.vertices.clear();
        self.indices.clear();
    }
}
