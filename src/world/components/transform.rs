use crate::renderer::buffer::BufferElem;

pub struct Transform {
    position: glam::Vec3,
    rotation: glam::Quat,
    scale: glam::Vec3,
    dirty: bool,
}

impl Transform {
    pub fn origin() -> Transform {
        Transform { 
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
            dirty: true,
        }
    }

    pub fn at(self, at: glam::Vec3) -> Transform {
        Transform {
            position: at,
            dirty: true,
            ..self
        }
    }

    pub fn rotated(self, rotation: glam::Quat) -> Transform {
        Transform { 
            rotation,
            dirty: true,
            ..self
        }
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn set_clean(&mut self) {
        self.dirty = false;
    }

    pub(crate) fn recompute_matrix(&self) -> TransformToGpu {
        TransformToGpu::new(
            glam::Mat4::from_scale_rotation_translation(
                self.scale,
                self.rotation,
                self.position
            )
        )
    }

}


#[derive(Debug, Clone, Copy)]
pub(crate) struct TransformToGpu {
    #[allow(unused)]
    model_mat: glam::Mat4,
    #[allow(unused)]
    inv_model: glam::Mat4,
}

unsafe impl bytemuck::Zeroable for TransformToGpu {}
unsafe impl bytemuck::Pod for TransformToGpu {}

const TRANSFORM_SIZE: usize = std::mem::size_of::<TransformToGpu>();

impl BufferElem for TransformToGpu {
    const BINDING: u32 = 0;
    const BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: true,
        min_binding_size: None,
    };
    #[cfg(debug_assertions)]
    const LABEL: &'static str = "transform";
    const VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    const SIZE: u64 = TRANSFORM_SIZE as u64;
    fn to_bytes(&self) -> &[u8] {
        bytemuck::cast_ref::<TransformToGpu, [u8; TRANSFORM_SIZE]>(self)
    }
}

impl TransformToGpu {
    pub(crate) fn new(model_mat: glam::Mat4) -> TransformToGpu {
        TransformToGpu {
            model_mat,
            inv_model: model_mat.inverse()
        }
    }
}