use legion::IntoQuery;

use crate::world::{components::{transform::Transform, csg_renderer::CsgRenderer}, camera::Camera};

use super::{csg_buffer::CsgBuffer, shader_data::ShaderData, screen_resolution::ScreenResolution};


/// WGPU stuff. Includes unsafe references to the created surface,
/// so we need to be careful with this.
/// I would like to involve a lifetime in there, but then I have more issues in examples.
/// I'll figure it out later.
/// Maybe a Renderer generic over the surface, and provide a way to get it back when needed?
pub struct RenderingState {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: (u32, u32),
    pub(crate) screen_resolution: ScreenResolution,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl RenderingState {
    pub fn new<T>(handle: &T, start_size: (u32, u32)) -> Result<RenderingState, crate::error::MorpheusError>
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

        let pipeline = Self::create_pipeline(&device, &config);

        let screen_resolution = ScreenResolution::new(&device, start_size);

        Ok(RenderingState {
            surface,
            device,
            queue,
            config,
            size: start_size,
            pipeline,
            screen_resolution,
        })
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.screen_resolution.resize(&self.queue, new_size);
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&self, world: &crate::world::World) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let mut query = <(&Transform, &CsgRenderer)>::query();

        for (_transform, csg_renderer) in query.iter(world.legion_world()) {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.6,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &world.main_camera().bind_group(), &[]);
            render_pass.set_bind_group(1, &self.screen_resolution.bind_group, &[]);
            render_pass.set_bind_group(2, &csg_renderer.bind_group(), &[]);
            render_pass.draw(0..36, 0..1);
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn create_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/raymarcher.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main Render Pipeline Layout"),
            bind_group_layouts: &[
                &Camera::bind_group_layout(device),
                &ScreenResolution::bind_group_layout(device),
                &CsgBuffer::bind_group_layout(device),
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        render_pipeline
    }
}