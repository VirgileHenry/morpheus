pub mod rendering_state;


pub struct Renderer {
    state: self::rendering_state::RenderingState,
}


impl Renderer {
    pub fn new<T>(handle: &T, start_size: (u32, u32)) -> Result<Renderer, crate::error::MorpheusError>
        where T: wgpu::raw_window_handle::HasRawWindowHandle + wgpu::raw_window_handle::HasRawDisplayHandle,
    {
        Ok(Renderer {
            state: rendering_state::RenderingState::new(handle, start_size)?,
        })
    }

    pub fn state(&self) -> &self::rendering_state::RenderingState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut self::rendering_state::RenderingState {
        &mut self.state
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        self.state.render()
    }
}