use wgpu::util::DeviceExt;

use super::shader_data::HasBindGroupLayout;

/// trait for any types we need to put into a buffer.
/// This provides info on how we are going to bind that type in the buffer.
/// It can feel weird to be strict on where types are bound in the gpu,
/// but in the context of building a renderer (and not an abstraction for renderers)
/// when I'm creating my types for buffers I create them for specific bindings.
/// This is what is abstracted here. But I guess this abstraction is discutable ?
pub(crate) trait BufferElem : bytemuck::Pod {
    const VISIBILITY: wgpu::ShaderStages;
    const BINDING: u32;
    const BINDING_TYPE: wgpu::BindingType;
    #[cfg(debug_assertions)]
    const LABEL: &'static str;
    fn to_bytes(&self) -> &[u8];
}

/// Abstraction over wgpu buffer, where we hold data of type T.
/// If type array is true, this is a buffer over an array of T.
/// otherwise, the buffer will hold a single T.
/// T will be sent as POD on the gpu.
/// 
/// ### Safety:
/// this assumes the data T can be transparently sent to the gpu.
pub(crate) struct Buffer<T: BufferElem, const TYPE_ARRAY: bool> {
    marker: std::marker::PhantomData<T>,
    #[allow(unused)]
    buffer_size: usize, // not used if !TYPE_ARRAY
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl<T: BufferElem> Buffer<T, false> {
    pub(crate) fn new(device: &wgpu::Device, t: T) -> Buffer<T, false> {

        let buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: t.to_bytes(),
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
            label: Some("camera bind group"),
        });

        Buffer { 
            marker: Default::default(),
            buffer_size: 1,
            buffer,
            bind_group,
        }
    }

    pub(crate) fn update(&mut self, queue: &wgpu::Queue, t: T) {
        queue.write_buffer(&self.buffer, 0, t.to_bytes())
    }
}

impl<T: BufferElem, const TYPE_ARRAY: bool> Buffer<T, TYPE_ARRAY> {
    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<T: BufferElem, const TYPE_ARRAY: bool> HasBindGroupLayout for Buffer<T, TYPE_ARRAY> {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        #[cfg(debug_assertions)]
        let label = format!("{:?} buffer bind group layout", T::LABEL);
        #[cfg(debug_assertions)]
        let label = Some(label.as_str());
        #[cfg(not(debug_assertions))]
        let label = Some("buffer bind group layout");
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: T::BINDING,
                    visibility: T::VISIBILITY,
                    ty: T::BINDING_TYPE,
                    count: None,
                }
            ],
            label,
        })
    }
}


