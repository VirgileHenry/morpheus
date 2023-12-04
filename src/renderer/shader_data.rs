

/// Trait for objects that want to be used in a shader.
pub(crate) trait ShaderData {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
}