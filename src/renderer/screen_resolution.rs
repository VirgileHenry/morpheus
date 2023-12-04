use super::shader_data::ShaderData;
use wgpu::util::DeviceExt;

pub struct ScreenResolution {
    buffer: wgpu::Buffer, // width: u32, height: u32
    pub(crate) bind_group: wgpu::BindGroup,
}

impl ScreenResolution {
    pub fn new(device: &wgpu::Device, screen_size: (u32, u32)) -> ScreenResolution {
        let buffer_content = size_to_buffer(screen_size);

        let buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };
        
        let buffer = device.create_buffer_init(&buffer_init);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("screen resolution bind group"),
        });

        ScreenResolution {
            buffer,
            bind_group,
        }
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, new_size: (u32, u32)) {
        let data = size_to_buffer(new_size);
        queue.write_buffer(&self.buffer, 0, &data);
    }
}

impl ShaderData for ScreenResolution {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("screen resolution bind group layout"),
        })
    }
}

fn size_to_buffer(size: (u32, u32)) -> [u8; 8] {
    let mut buffer = [0u8; 8];
    let bytes = [
        size.0.to_ne_bytes(),
        size.1.to_ne_bytes(),
    ];
    for (i, byte) in bytes.into_iter().flatten().enumerate() {
        buffer[i] = byte;
    }
    buffer
}