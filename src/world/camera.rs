use wgpu::util::DeviceExt;

use crate::renderer::shader_data::ShaderData;

pub struct Camera {
    _position: glam::Vec3,
    _view_dir: glam::Quat,
    _fovy: f32,
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(device: &wgpu::Device, at: glam::Vec3) -> Camera {
        let position = at;
        let view_dir = glam::Quat::IDENTITY;
        let fovy = 1.5;

        let buffer_content = buffer_from_cam(position, view_dir, fovy);

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
            label: Some("camera bind group"),
        });

        Camera {
            _position: position,
            _view_dir: view_dir,
            _fovy: fovy,
            _buffer: buffer,
            bind_group,
        }

    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl ShaderData for Camera {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera bind group layout"),
        })
    }
}

const CAMERA_GPU_SIZE: usize = 128; // mat4 is 64

fn buffer_from_cam(position: glam::Vec3, view_dir: glam::Quat, fovy: f32) -> [u8; CAMERA_GPU_SIZE] {
    let view_mat = glam::Mat4::from_rotation_translation(view_dir, position);
    let proj_mat = glam::Mat4::perspective_infinite_rh(fovy, 1., 0.);
    let view_proj = proj_mat * view_mat.inverse();
    let inv_proj = proj_mat.inverse();
    let mut buffer = [0u8; CAMERA_GPU_SIZE];

    for (i, value) in view_proj.as_ref().into_iter().enumerate() {
        for (j, byte) in value.to_ne_bytes().into_iter().enumerate() {
            buffer[std::mem::size_of::<f32>() * i + j] = byte;
        }
    }
    for (i, value) in inv_proj.as_ref().into_iter().enumerate() {
        for (j, byte) in value.to_ne_bytes().into_iter().enumerate() {
            buffer[std::mem::size_of::<glam::Mat4>() + std::mem::size_of::<f32>() * i + j] = byte;
        }
    }

    buffer
}