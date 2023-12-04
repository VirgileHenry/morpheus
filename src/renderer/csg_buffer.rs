use csg::csg_traits::node_iter::NodeIter;
use csg::csg_traits::binarize::BinarizeCsgTree;
use csg::csg_traits::csg_tree_size::CsgTreeSize;
use wgpu::util::DeviceExt;

use super::rendering_state::RenderingState;
use super::shader_data::ShaderData;

/// WGPU buffer that contains a csg object.
pub(crate) struct CsgBuffer {
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl CsgBuffer {

    pub(crate) fn new(wgpu_state: &RenderingState, csg: &csg::csg_object::Object) -> CsgBuffer {
        // create the data for the csg object
        let buffer_content = match csg.clone().binarize() {
            Some(binarized) => {
                let size = binarized.size();
                let mut buffer = Vec::with_capacity(size.get() * CSG_NODE_GPU_SIZE);
                for node in binarized.nodes() {
                    let buffered_node = to_gpu_data(node);
                    buffer.extend_from_slice(&buffered_node);
                }
                buffer
            },
            None => vec![0u8; CSG_NODE_GPU_SIZE], // single node with id 0 represent empty obj
        };

        let buffer_init = wgpu::util::BufferInitDescriptor {
            label: Some("CSG Object data buffer"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        };
        
        let buffer = wgpu_state.device.create_buffer_init(&buffer_init);

        let bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(&wgpu_state.device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        
        CsgBuffer {
            _buffer: buffer,
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
                }
            ],
            label: Some("csg buffer bind group layout"),
        })
    }
}

/// Number of bytes a csg node takes on the gpu
/// Size of the node for now, maybe we can squeeze this manually later on
/// Should be a multiple of 16 for alignment 
const CSG_NODE_GPU_SIZE: usize = 32; //std::mem::size_of::<csg::csg_node::Node>(); 

fn to_gpu_data(node: csg::csg_node::Node) -> [u8; CSG_NODE_GPU_SIZE] {
    let mut result = [0u8; CSG_NODE_GPU_SIZE];
    let as_slice = result.as_mut_slice();
    for (i, byte) in node.id().to_ne_bytes().into_iter().enumerate() {
        as_slice[i] = byte
    }

    match node {
        // todo : put data when necessary
        csg::csg_node::Node::PrimitiveSphere { center, radius } => {
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