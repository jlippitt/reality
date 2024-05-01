use super::{Texture, TileAddressMode};
use crate::gfx::GfxContext;
use bytemuck::{Pod, Zeroable};
use std::rc::Rc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct TileViewAxis {
    pub clamp: u32,
    pub mirror: u32,
    pub mask: u32,
    pub shift: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct TileViewUniform {
    s: TileViewAxis,
    t: TileViewAxis,
}

pub struct TileViewOptions<'a> {
    pub gfx: &'a GfxContext,
    pub sampler: &'a wgpu::Sampler,
    pub bind_group_layout: &'a wgpu::BindGroupLayout,
    pub texture: Rc<Texture>,
    pub address_s: &'a TileAddressMode,
    pub address_t: &'a TileAddressMode,
}

#[derive(Debug)]
pub struct TileView {
    bind_group: wgpu::BindGroup,
    _texture: Rc<Texture>,
    _uniform_buffer: wgpu::Buffer,
}

impl TileView {
    pub fn new(options: TileViewOptions<'_>) -> Self {
        let TileViewOptions {
            gfx,
            sampler,
            bind_group_layout,
            texture,
            address_s,
            address_t,
        } = options;

        let uniform = TileViewUniform {
            s: address_s.clone().into(),
            t: address_t.clone().into(),
        };

        let uniform_buffer = gfx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = gfx.device().create_bind_group({
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            }
        });

        Self {
            bind_group,
            _texture: texture,
            _uniform_buffer: uniform_buffer,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl From<TileAddressMode> for TileViewAxis {
    fn from(value: TileAddressMode) -> Self {
        Self {
            clamp: value.clamp as u32,
            mirror: value.mirror as u32,
            mask: value.mask,
            shift: value.shift,
        }
    }
}
