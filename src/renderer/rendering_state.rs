use super::deferred_renderer::DeferredRenderer;


/// WGPU stuff. Includes unsafe references to the created surface,
/// so we need to be careful with this.
/// I would like to involve a lifetime in there, but then I have more issues in examples.
/// I'll figure it out later.
/// Maybe a Renderer generic over the surface, and provide a way to get it back when needed?
pub(crate) struct RenderingState {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: (u32, u32),
    pub(crate) renderer: DeferredRenderer,
}

impl RenderingState {
    pub(crate) fn new<T>(handle: &T, start_size: (u32, u32)) -> Result<RenderingState, crate::error::MorpheusError>
        where T: wgpu::raw_window_handle::HasRawWindowHandle + wgpu::raw_window_handle::HasRawDisplayHandle,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL, // any other does not work yet for me :( 
            ..Default::default()
        });
    
        let surface = unsafe { instance.create_surface(handle) }.unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).ok_or(crate::error::MorpheusError::NoAvailableAdapter)?;
        
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ))?;
    
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())            
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: start_size.0,
            height: start_size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let renderer = DeferredRenderer::new(&device, &config);

        Ok(RenderingState {
            surface,
            device,
            queue,
            config,
            size: start_size,
            renderer,
        })
    }

    pub(crate) fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.renderer.resize(&self.device, &self.queue, new_size);
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub(crate) fn update_uniforms(&mut self, world: &mut crate::world::World) {
        self.renderer.update_uniforms(world, &self.queue)
    }

    pub(crate) fn render(&self, world: &crate::world::World) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(world, &self.device, &self.queue, &self.surface)
    }

}