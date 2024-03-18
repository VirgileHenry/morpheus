use crate::world::{camera::Camera, components::transform::Transform};

use self::{asset_manager::AssetManager, assets::csg::CsgObjectAsset};

pub(crate) mod assets;
pub(crate) mod asset_manager;
pub(crate) mod buffer;
pub(crate) mod deferred_renderer;
pub(crate) mod has_bind_group_layout;
pub(crate) mod rendering_state;
pub(crate) mod screen_resolution;


/// Central morpheus app renderer.
/// This include the wgpu state, as well as the world with entities and components.
pub struct Renderer {
    state: self::rendering_state::RenderingState,
    assets: AssetManager,
    world: crate::world::World,
}

impl Renderer {
    pub fn new<T>(handle: &T, start_size: (u32, u32)) -> Result<Renderer, crate::error::MorpheusError>
        where T: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let state = rendering_state::RenderingState::new(handle, start_size)?;
        let main_camera = Camera::new(&state.device, glam::vec3(0., 0.4, 2.0), start_size);
        let world = crate::world::World::new(main_camera);

        let assets = AssetManager::new();

        Ok(Renderer {
            state,
            assets,
            world,
        })
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.state.resize(new_size);
        self.world.main_camera_mut().viewport_resize(&self.state.queue, new_size);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.state.update_uniforms(&mut self.world);
        // check world rebuild
        if self.assets.dirty() {
            self.assets.reload(&self.state.device, &self.state.queue);
        }
        self.state.render(&mut self.world, &self.assets)
    }

    pub fn load_csg(&mut self, asset_id: u64, csg: csg::CSG) {
        let asset = CsgObjectAsset::new(&self.state.device, csg);
        self.assets.load(asset_id, asset);
    }

    pub fn create_obj(&mut self, transform: Transform, asset_id: u64) {
        self.world.add_obj(transform, asset_id);
    }
}