pub mod components;
pub(crate) mod camera;

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

    pub(crate) fn add_obj(&mut self, transform: Transform, csg_asset_id: u64) {
        let renderer = CsgRenderer::new(csg_asset_id);
        self.world.push((transform, renderer));
    }

}