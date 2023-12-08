use crate::renderer::{csg_buffer::CsgBuffer, rendering_state::RenderingState};



pub(crate) struct CsgRenderer {
    csg: csg::object::Object,
    _bounding_box_size: f32,
    buffer: CsgBuffer, // the buffer of csg tree like on gpu
    dirty: bool, // buffer need to be rebuilt
}

impl CsgRenderer {
    pub fn new(wgpu_state: &RenderingState, csg: csg::object::Object) -> CsgRenderer {
        
        let buffer = CsgBuffer::new(wgpu_state, &csg);

        CsgRenderer {
            csg,
            _bounding_box_size: 1., // todo: compute
            buffer,
            dirty: false,
        }
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        self.buffer.bind_group()
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }
 
    pub(crate) fn update_csg_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.buffer.update_csg(device, queue, &self.csg);
        self.dirty = false
    }
}

