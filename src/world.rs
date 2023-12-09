pub mod components;
pub(crate) mod camera;

use legion::IntoQuery;

use crate::renderer::rendering_state::RenderingState;

use self::{camera::Camera, components::{transform::Transform, csg_renderer::CsgRenderer}};


/// Representation of the world we are trying to render.
pub struct World {
    world: legion::World,
    main_camera: Camera,
    dirty: bool, // needs rebuild on some components
}

impl World {
    pub fn new(main_camera: Camera) -> World {
        World {
            world: legion::World::default(),
            main_camera,
            dirty: false,
        }
    }

    pub(crate) fn legion_world(&self) -> &legion::World {
        &self.world
    }

    pub(crate) fn legion_world_mut(&mut self) -> &mut legion::World {
        &mut self.world
    }

    pub(crate) fn main_camera(&self) -> &Camera {
        &self.main_camera
    }

    pub(crate) fn main_camera_mut(&mut self) -> &mut Camera {
        &mut self.main_camera
    }

    pub(crate) fn add_obj(&mut self, transform: Transform, csg: csg::object::Object, state: &RenderingState) {
        let renderer = CsgRenderer::new(state, csg);
        self.world.push((transform, renderer));
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn rebuild(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut query = <&mut CsgRenderer as IntoQuery>::query();
        for csg_renderer in query.iter_mut(&mut self.world) {
            if csg_renderer.is_dirty() {
                csg_renderer.update_csg_buffer(device, queue);
            }
        }
        self.dirty = false;
    }
    
}