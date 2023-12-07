use wgpu::util::DeviceExt;

use crate::renderer::shader_data::ShaderData;

pub(super) struct StorageBuffer<T: bytemuck::Pod> {
    marker: std::marker::PhantomData<T>,
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}


impl<T: bytemuck::Pod> StorageBuffer<T> {
    pub(super) fn new(device: &wgpu::Device, init_size: usize) -> StorageBuffer<T> {
        // create the data for the csg object
        let buffer_content = vec![0u8; init_size * std::mem::size_of::<T>()];

        let buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        };
        
        let buffer = device.create_buffer_init(&buffer_init);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(&device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("generic storage buffer bind group"),
        });
        
        StorageBuffer {
            marker: Default::default(),
            _buffer: buffer,
            bind_group,
        }
    } 

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<T: bytemuck::Pod> ShaderData for StorageBuffer<T> {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("storage buffer bind group layout"),
        })
    }
}

