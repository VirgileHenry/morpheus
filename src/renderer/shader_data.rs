

/// Trait for objects that provid a bind group layout.
/// This allow to not duplicate bind groups layouts and minimize redondant code
/// as well as mistakes from non matching bind groups
pub(crate) trait HasBindGroupLayout {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
}