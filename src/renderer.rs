use crate::world::{camera::Camera, components::transform::Transform};

pub(crate) mod csg_buffer;
pub(crate) mod rendering_state;
pub(crate) mod screen_resolution;
pub(crate) mod shader_data;
pub(crate) mod deffered_renderer;


/// Central morpheus app renderer.
/// This include the wgpu state, as well as the world with entities and components.
pub struct Renderer {
    state: self::rendering_state::RenderingState,
    world: crate::world::World,
}

impl Renderer {
    pub fn new<T>(handle: &T, start_size: (u32, u32)) -> Result<Renderer, crate::error::MorpheusError>
        where T: wgpu::raw_window_handle::HasRawWindowHandle + wgpu::raw_window_handle::HasRawDisplayHandle,
    {
        let state = rendering_state::RenderingState::new(handle, start_size)?;
        let main_camera = Camera::new(&state.device, glam::vec3(0., 0.4, 2.0), start_size);
        let world = crate::world::World::new(main_camera);
        Ok(Renderer {
            state,
            world,
        })
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.state.resize(new_size);
        self.world.main_camera_mut().viewport_resize(&self.state.queue, new_size);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // check world rebuild
        if self.world.is_dirty() {
            self.world.rebuild(&self.state.device, &self.state.queue);
        }
        self.state.render(&mut self.world)
    }

    pub fn create_obj(&mut self, csg: csg::object::Object) {
        self.world.add_obj(Transform::origin(), csg, &self.state);
    }
}