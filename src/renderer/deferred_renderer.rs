mod texture;
mod textures;
// mod storage_buffer;

use legion::IntoQuery;
use self::textures::{AlbedoTexture, NormalDepthTexture};

use super::buffer::Buffer;
use super::{
    screen_resolution::ScreenResolution,
    csg_buffer::CsgBuffer
};
use crate::world::camera::CameraToGpu;
use crate::renderer::has_bind_group_layout::HasBindGroupLayout;
use crate::world::components::csg_renderer::CsgRenderer;
use crate::world::components::transform::Transform;



/// A deffered renderer.
pub(crate) struct DeferredRenderer {
    first_stage_pipeline: wgpu::RenderPipeline,
    second_stage_pipeline: wgpu::RenderPipeline,
    screen_resolution: Buffer<ScreenResolution, false>,
    albedo_tex: self::texture::Texture<AlbedoTexture>,
    normal_depth_tex: self::texture::Texture<NormalDepthTexture>,
}

impl DeferredRenderer {
    pub(crate) fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> DeferredRenderer {
        let first_stage_pipeline = create_first_stage_pipeline(device);
        let second_stage_pipeline = create_second_stage_pipeline(device, config);
        let screen_resolution = Buffer::<ScreenResolution, false>::new(&device, ScreenResolution::new(config.width, config.height));

        let size = (config.width, config.height);
        
        let albedo_tex = self::texture::Texture::new(
            device, size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let normal_depth_tex = self::texture::Texture::new(
            device, size,
            wgpu::TextureFormat::Rgba16Float,
        );

        DeferredRenderer { 
            first_stage_pipeline,
            second_stage_pipeline,
            screen_resolution,
            albedo_tex,
            normal_depth_tex,
        }
    }

    pub(crate) fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, new_size: (u32, u32)) {
        self.screen_resolution.update(queue, ScreenResolution::new(new_size.0, new_size.1));
        // resize all temps textures
        self.albedo_tex.resize(device, new_size);
        self.normal_depth_tex.resize(device, new_size);
    }

    pub(crate) fn render(&self, world: &crate::world::World, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;

        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let albedo_view = self.albedo_tex.get_view();
        let normal_depth_view = self.normal_depth_tex.get_view();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("deferred renderer encoder"),
        });
        
        let color_attachments = [
            Some(wgpu::RenderPassColorAttachment {
                view: &albedo_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0, g: 0.0, b: 0.0, a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &normal_depth_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0, g: 0.0, b: 0.0, a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            }),
        ];

        let mut first_stage_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("first stage render pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        first_stage_render_pass.set_pipeline(&self.first_stage_pipeline);
        first_stage_render_pass.set_bind_group(0, &world.main_camera().bind_group(), &[]);
        first_stage_render_pass.set_bind_group(1, &self.screen_resolution.bind_group(), &[]);

        let mut query = <(&Transform, &CsgRenderer)>::query();
        for (_transform, csg_renderer) in query.iter(world.legion_world()) {
            first_stage_render_pass.set_bind_group(2, &csg_renderer.bind_group(), &[]);
            // draw the hard coded bounding box
            first_stage_render_pass.draw(0..36, 0..1);
        }

        drop(first_stage_render_pass);
        
        let mut second_stage_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("second stage render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0, g: 0.0, b: 0.0, a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // second stage
        second_stage_render_pass.set_pipeline(&self.second_stage_pipeline);
        second_stage_render_pass.set_bind_group(0, &self.screen_resolution.bind_group(), &[]);
        second_stage_render_pass.set_bind_group(1, self.albedo_tex.bind_group(), &[]);
        second_stage_render_pass.set_bind_group(2, self.normal_depth_tex.bind_group(), &[]);
        // draw the hard coded quad
        second_stage_render_pass.draw(0..6, 0..1);
        
        drop(second_stage_render_pass);

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn create_first_stage_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/raymarcher.wgsl"));
        
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("first stage pipeline layout"),
        bind_group_layouts: &[
            &Buffer::<CameraToGpu, false>::bind_group_layout(device),
            &Buffer::<ScreenResolution, false>::bind_group_layout(device),
            &CsgBuffer::bind_group_layout(device),
        ],
        push_constant_ranges: &[],
    });

    let fragment_target = [
        // albedo target
        Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }),
        // normals and depth (rbg is normal, a is depth)
        Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Rgba16Float,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }),
        // to add : material props ? velocity ?
    ];
    
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("first stage render pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &fragment_target,
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
    })
}


fn create_second_stage_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/deferred_lighting.wgsl"));
        
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("second stage pipeline layout"),
        bind_group_layouts: &[
            &Buffer::<ScreenResolution, false>::bind_group_layout(device),
            &self::texture::Texture::<AlbedoTexture>::bind_group_layout(device),
            &self::texture::Texture::<NormalDepthTexture>::bind_group_layout(device),
        ],
        push_constant_ranges: &[],
    });
    
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("second stage render pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
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
    })
}