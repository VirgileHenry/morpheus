pub(crate) mod csg_buffer;

use crate::renderer::asset_manager::asset::AssetTrait;

use self::csg_buffer::CsgBuffer;



pub struct CsgObjectAsset {
    buffer: CsgBuffer,
    csg: csg::CSG,
}

impl CsgObjectAsset {

    pub fn new(device: &wgpu::Device, csg: csg::CSG) -> CsgObjectAsset {
        let buffer = CsgBuffer::new(device, &csg);

        CsgObjectAsset {
            buffer,
            csg,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.buffer.bind_group()
    }
}

impl AssetTrait for CsgObjectAsset {
    fn relaod(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.buffer.update_csg(device, queue, &self.csg);
    }
}