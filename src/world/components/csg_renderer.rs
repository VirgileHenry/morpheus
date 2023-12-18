
pub(crate) struct CsgRenderer {
    _bounding_box_size: f32,
    csg_asset_id: u64,
}

impl CsgRenderer {
    pub fn new(asset_id: u64) -> CsgRenderer {
        CsgRenderer {
            _bounding_box_size: 1., // todo: compute
            csg_asset_id: asset_id,
        }
    }

    pub(crate) fn asset_id(&self) -> u64 {
        self.csg_asset_id
    } 

}

