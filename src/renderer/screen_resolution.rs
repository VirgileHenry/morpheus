use super::buffer::BufferElem;


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScreenResolution {
    width: u32,
    height: u32,
}

unsafe impl bytemuck::Zeroable for ScreenResolution {}
unsafe impl bytemuck::Pod for ScreenResolution {}

const SREEN_RESOLUTION_SIZE: usize = 8;

impl BufferElem for ScreenResolution {
    const BINDING: u32 = 0;
    const BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    };
    const LABEL: &'static str = "screen resolution";
    const VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    fn to_bytes(&self) -> &[u8] {
        bytemuck::cast_ref::<ScreenResolution, [u8; SREEN_RESOLUTION_SIZE]>(&self)
    }
}

impl ScreenResolution {
    pub fn new(width: u32, height: u32) -> ScreenResolution {
        ScreenResolution {
            width,
            height,
        }
    }
}
