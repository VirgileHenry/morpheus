pub(crate) mod components;
pub(crate) mod camera;

use crate::renderer::rendering_state::RenderingState;

use self::{camera::Camera, components::{transform::Transform, csg_renderer::CsgRenderer}};


/// Representation of the world we are trying to render.
pub struct World {
    world: legion::World,
    main_camera: Camera,
}

impl World {
    pub fn new(main_camera: Camera) -> World {
        World {
            world: legion::World::default(),
            main_camera,
        }
    }

    pub fn legion_world(&self) -> &legion::World {
        &self.world
    }

    pub fn main_camera(&self) -> &Camera {
        &self.main_camera
    }

    pub fn add_obj(&mut self, transform: Transform, csg: csg::csg_object::Object, state: &RenderingState) {
        let renderer = CsgRenderer::new(state, csg);
        self.world.push((transform, renderer));
    }
    
}