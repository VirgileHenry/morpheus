use std::num::NonZeroUsize;

use csg::traits::node_iter::NodeIter;
use csg::traits::binarize::BinarizeCsgTree;
use csg::traits::tree_size::TreeSize;
use csg::traits::tree_height::TreeHeight;
use wgpu::util::DeviceExt;

use super::rendering_state::RenderingState;
use super::shader_data::ShaderData;

/// WGPU buffer that contains a csg object.
pub(crate) struct CsgBuffer {
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl CsgBuffer {

    pub(crate) fn new(wgpu_state: &RenderingState, csg: &csg::object::Object) -> CsgBuffer {
        // create the data for the csg object
        let (csg_buffer_content, _sdf_stack_size, csg_full_size) = match csg.clone().binarize() {
            Some(binarized) => {
                let size = binarized.size();
                let height = binarized.height();
                let mut buffer = Vec::with_capacity(size.get() * CSG_NODE_GPU_SIZE);
                let nodes = binarized.nodes().collect::<Vec<_>>();
                for node in nodes.into_iter().rev() {
                    let buffered_node = to_gpu_data(node);
                    buffer.extend_from_slice(&buffered_node);
                }
                (buffer, height, size)
            },
            None => (vec![0u8; CSG_NODE_GPU_SIZE], unsafe { NonZeroUsize::new_unchecked(1) }, unsafe { NonZeroUsize::new_unchecked(1) }), // single node with id 0 represent empty obj
        };

        let csg_buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: &csg_buffer_content,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        };
        let csg_size_buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: &csg_full_size.get().to_ne_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };

        let csg_buffer = wgpu_state.device.create_buffer_init(&csg_buffer_init);
        let csg_size_buffer = wgpu_state.device.create_buffer_init(&csg_size_buffer_init);

        let bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(&wgpu_state.device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: csg_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: csg_size_buffer.as_entire_binding(),
                },
            ],
            label: Some("csg buffer bind group"),
        });
        
        CsgBuffer {
            _buffer: csg_buffer,
            bind_group,
        }
    } 

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl ShaderData for CsgBuffer {
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
const CSG_NODE_GPU_SIZE: usize = 32; //std::mem::size_of::<csg::csg_node::Node>(); 

fn to_gpu_data(node: csg::node::Node) -> [u8; CSG_NODE_GPU_SIZE] {
    let mut result = [0u8; CSG_NODE_GPU_SIZE];
    let as_slice = result.as_mut_slice();
    for (i, byte) in node.id().to_ne_bytes().into_iter().enumerate() {
        as_slice[i] = byte
    }

    match node {
        // todo : put data when necessary
        csg::node::Node::PrimitiveSphere { center, radius } => {
            let bytes = [
                center.x.to_ne_bytes(),
                center.y.to_ne_bytes(),
                center.z.to_ne_bytes(),
                radius.to_ne_bytes(),
            ];
            for (i, byte) in bytes.into_iter().flatten().enumerate() {
                result[4 + i] = byte;
            }
        }
        _ => { /* no data to pass */ }
    }

    result
}