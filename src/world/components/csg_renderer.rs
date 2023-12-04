use crate::renderer::{csg_buffer::CsgBuffer, rendering_state::RenderingState};



pub(crate) struct CsgRenderer {
    _csg: csg::csg_object::Object,
    _bounding_box_size: f32,
    buffer: CsgBuffer, // the buffer of csg tree like on gpu
}

impl CsgRenderer {
    pub fn new(wgpu_state: &RenderingState, csg: csg::csg_object::Object) -> CsgRenderer {
        
        let buffer = CsgBuffer::new(wgpu_state, &csg);

        CsgRenderer {
            _csg: csg,
            _bounding_box_size: 1., // todo: compute
            buffer,
        }
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        self.buffer.bind_group()
    }
}

