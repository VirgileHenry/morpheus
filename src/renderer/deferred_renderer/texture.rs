use crate::renderer::has_bind_group_layout::HasBindGroupLayout;


/// trait that provides informations about any textures
/// that will be used by the deferred shader.
/// This allow to slightly change the behaviour of the texture abstraction struct
/// depending on the use of the texture
pub(super) trait TextureTypeInfo {
    #[cfg(debug_assertions)]
    const LABEL: &'static str;
}

/// Texture interface for the deferred renderer.
pub(super) struct Texture<T: TextureTypeInfo> {
    marker: std::marker::PhantomData<T>,
    format: wgpu::TextureFormat,
    texture: wgpu::Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl<T: TextureTypeInfo> Texture<T> {
    pub fn new(device: &wgpu::Device, size: (u32, u32), format: wgpu::TextureFormat) -> Texture<T> {
        
        let descriptor = wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.0, height: size.1, depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        };
        
        let texture = device.create_texture(&descriptor);
        let bind_group_layout = Self::bind_group_layout(device);

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &texture.create_view(&wgpu::TextureViewDescriptor::default())
                        ),
                    },
                ],
                label: Some("texture bind group"),
            }
        );

        Texture {
            marker: Default::default(),
            format,
            texture,
            bind_group_layout,
            bind_group,
        }
    }

    pub(super) fn get_view(&self) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub(super) fn resize(&mut self, device: &wgpu::Device, new_size: (u32, u32)) {
        let descriptor = wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: new_size.0, height: new_size.1, depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[self.format],
        };
        self.texture = device.create_texture(&descriptor);

        #[cfg(debug_assertions)]
        let label = format!("{:?} texture bind group", <T as TextureTypeInfo>::LABEL);
        #[cfg(debug_assertions)]
        let label = Some(label.as_str());
        #[cfg(not(debug_assertions))]
        let label = Some("texture bind group");

        self.bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &self.texture.create_view(&wgpu::TextureViewDescriptor::default())
                        ),
                    },
                ],
                label,
            }
        );
    }

    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<T: TextureTypeInfo> HasBindGroupLayout for Texture<T> {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        #[cfg(debug_assertions)]
        let label = format!("{:?} texture bind group layout", <T as TextureTypeInfo>::LABEL);
        #[cfg(debug_assertions)]
        let label = Some(label.as_str());
        #[cfg(not(debug_assertions))]
        let label = Some("texture bind group layout");
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
            label,
        })
    }
}
