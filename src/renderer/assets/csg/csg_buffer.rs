use wgpu::util::DeviceExt;

use crate::renderer::has_bind_group_layout::HasBindGroupLayout;


/// WGPU buffer that contains a csg object.
pub(crate) struct CsgBuffer {
    buffer_size: usize,
    buffer: wgpu::Buffer,
    size_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl CsgBuffer {

    pub(crate) fn new(device: &wgpu::Device, csg: &csg::CSG) -> CsgBuffer {

        let mut buffer = Vec::with_capacity(csg.node_count() * CSG_NODE_GPU_SIZE);

        // the gpu needs the csg vec in reverse to compute sdf
        for node in csg.nodes().rev() {
            let buffered_node = to_gpu_data(node);
            buffer.extend_from_slice(&buffered_node);
        }

        let node_count_u32: u32 = csg.node_count().try_into().expect("Unable to convert csg tree size to u32 !");

        let buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: &buffer,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        };
        let size_buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: &node_count_u32.to_ne_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };

        let buffer = device.create_buffer_init(&buffer_init);
        let size_buffer = device.create_buffer_init(&size_buffer_init);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(&device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: size_buffer.as_entire_binding(),
                },
            ],
            label: Some("csg buffer bind group"),
        });
        
        CsgBuffer {
            buffer_size: csg.node_count(),
            buffer,
            size_buffer,
            bind_group,
        }
    }

    pub(crate) fn update_csg(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, csg: &csg::CSG) {
        
        let mut buffer = Vec::with_capacity(csg.node_count() * CSG_NODE_GPU_SIZE);

        for node in csg.nodes().rev() {
            let buffered_node = to_gpu_data(node);
            buffer.extend_from_slice(&buffered_node);
        }

        if self.buffer_size >= csg.node_count() {
            // need to reallocate the csg buffer
            let buffer_init = wgpu::util::BufferInitDescriptor {
                label: Some("CSG Object data buffer"),
                contents: &buffer,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            };
            self.buffer = device.create_buffer_init(&buffer_init);
            self.buffer_size = csg.node_count();
        }
        else {
            queue.write_buffer(&self.buffer, 0, &buffer);
        }

        let node_count_u32: u32 = csg.node_count().try_into().expect("Unable to convert csg tree size to u32 !");
        queue.write_buffer(&self.size_buffer, 0, &node_count_u32.to_ne_bytes());
    } 

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl HasBindGroupLayout for CsgBuffer {
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
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("csg buffer bind group layout"),
        })
    }
}

/// Number of bytes a csg node takes on the gpu
/// Size of the node for now, maybe we can squeeze this manually later on
/// Should be a multiple of 16 for alignment 
const CSG_NODE_GPU_SIZE: usize = 4 + 4 * 11; //std::mem::size_of::<csg::csg_node::Node>(); 

fn to_gpu_data(node: &csg::node::CsgNode) -> [u8; CSG_NODE_GPU_SIZE] {
    let mut result = [0u8; CSG_NODE_GPU_SIZE];

    for (i, byte) in node.id().to_ne_bytes().into_iter().enumerate() {
        result[i] = byte
    }

    match node {
        // put data when necessary
        csg::node::CsgNode::Primitive(primitive) => load_primitive_data(primitive, &mut result[4..]),
        _ => { /* no data to pass */ }
    }

    result
}

fn load_primitive_data(primitive: &csg::Primitive, buffer: &mut [u8]) {
    match primitive {
        csg::Primitive::Sphere { radius, offset } => {
            let bytes = [
                offset.x.to_ne_bytes(),
                offset.y.to_ne_bytes(),
                offset.z.to_ne_bytes(),
                radius.to_ne_bytes(),
            ];
            for (i, byte) in bytes.into_iter().flatten().enumerate() {
                buffer[i] = byte;
            }
        }
        csg::Primitive::Cube { offset, rotation, size } => {
            let bytes = [
                offset.x.to_ne_bytes(),
                offset.y.to_ne_bytes(),
                offset.z.to_ne_bytes(),
                rotation.x.to_ne_bytes(),
                rotation.y.to_ne_bytes(),
                rotation.z.to_ne_bytes(),
                rotation.w.to_ne_bytes(),
                size.x.to_ne_bytes(),
                size.y.to_ne_bytes(),
                size.z.to_ne_bytes(),
            ];
            for (i, byte) in bytes.into_iter().flatten().enumerate() {
                buffer[i] = byte;
            }
        }
    }
}