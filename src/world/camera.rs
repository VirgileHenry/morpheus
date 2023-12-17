use crate::renderer::buffer::{BufferElem, Buffer};

pub struct Camera {
    position: glam::Vec3,
    view_dir: glam::Quat,
    fovy: f32,
    buffer: Buffer<CameraToGpu, false>,
}

impl Camera {
    pub fn new(device: &wgpu::Device, at: glam::Vec3, viewport_size: (u32, u32)) -> Camera {
        let position = at;
        let view_dir = glam::Quat::from_axis_angle(glam::Vec3::X, -0.3);
        let fovy = 1.5;

        let aspect_ratio = if viewport_size.1 > 0 { viewport_size.0 as f32 / viewport_size.1 as f32 } else { 1.0 };

        let cam_to_gpu = CameraToGpu::new(view_dir, position, aspect_ratio, fovy);
        let buffer = Buffer::<CameraToGpu, false>::new(device, cam_to_gpu);

        Camera {
            position,
            view_dir,
            fovy,
            buffer,
        }

    }

    pub fn viewport_resize(&mut self, queue: &wgpu::Queue, new_size: (u32, u32)) {
        let aspect_ratio = if new_size.1 > 0 { new_size.0 as f32 / new_size.1 as f32 } else { 1.0 };

        let cam_to_gpu = CameraToGpu::new(self.view_dir, self.position, aspect_ratio, self.fovy);
        self.buffer.update(queue, cam_to_gpu);
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.buffer.bind_group()
    }
}

/// data that is sent to the gpu
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct CameraToGpu {
    proj_view: glam::Mat4,
    inv_rot: glam::Mat4,
    position: glam::Vec3,
    fovy: f32,
}

const CAMERA_GPU_SIZE: usize = std::mem::size_of::<CameraToGpu>();

unsafe impl bytemuck::Zeroable for CameraToGpu {}
unsafe impl bytemuck::Pod for CameraToGpu {}

impl BufferElem for CameraToGpu {
    const BINDING: u32 = 0;
    const BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    #[cfg(debug_assertions)]
    const LABEL: &'static str = "camera";
    const VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    const SIZE: u64 = CAMERA_GPU_SIZE as u64;
    fn to_bytes(&self) -> &[u8] {
        bytemuck::cast_ref::<CameraToGpu, [u8; CAMERA_GPU_SIZE]>(&self)
    }
}

impl CameraToGpu {
    pub(crate) fn new(view_dir: glam::Quat, position: glam::Vec3, aspect_ratio: f32, fovy: f32) -> CameraToGpu {
        
        let view_mat = glam::Mat4::from_rotation_translation(view_dir, position);
        let proj_mat = glam::Mat4::perspective_infinite_rh(fovy, aspect_ratio, 0.1);
        let proj_view = proj_mat * view_mat.inverse();
        let inv_rot = glam::Mat4::from_rotation_translation(view_dir.inverse(), glam::Vec3::ZERO);

        CameraToGpu { 
            proj_view,
            inv_rot,
            position,
            fovy
        }
    }
}