use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Globals {
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    viewport_width: u32,
    viewport_height: u32,
    dirty: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GlobalsUniform {
    view_matrix: [[f32; 4]; 4],
}

unsafe impl Pod for GlobalsUniform {}
unsafe impl Zeroable for GlobalsUniform {}

impl Globals {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout_label = create_debug_label!("Globals bind group layout");
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: layout_label.as_deref(),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let buffer_label = create_debug_label!("Globals buffer");
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label.as_deref(),
            size: std::mem::size_of::<GlobalsUniform>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_label = create_debug_label!("Globals bind group");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: bind_group_label.as_deref(),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<GlobalsUniform>() as u64),
                },
            }],
        });

        Self {
            layout,
            bind_group,
            buffer,
            viewport_width: 0,
            viewport_height: 0,
            dirty: true,
        }
    }

    pub fn set_resolution(&mut self, viewport_width: u32, viewport_height: u32) {
        if viewport_width != self.viewport_width || viewport_height != self.viewport_height {
            self.viewport_width = viewport_width;
            self.viewport_height = viewport_height;
            self.dirty = true;
        }
    }

    pub fn update_uniform(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        let temp_label = create_debug_label!("Temporary globals buffer");
        let temp_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: temp_label.as_deref(),
            contents: bytemuck::cast_slice(&[GlobalsUniform {
                view_matrix: [
                    [1.0 / (self.viewport_width as f32 / 2.0), 0.0, 0.0, 0.0],
                    [0.0, -1.0 / (self.viewport_height as f32 / 2.0), 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0],
                ],
            }]),
            usage: wgpu::BufferUsage::COPY_SRC,
        });

        encoder.copy_buffer_to_buffer(
            &temp_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of::<GlobalsUniform>() as u64,
        );
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
