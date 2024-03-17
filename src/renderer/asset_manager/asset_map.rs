

use std::collections::HashMap;

use super::asset::AssetTrait;


/// Map of assets of the type T.
pub(crate) struct AssetMap<T> where AssetMap<T>: AssetMapTrait {
    pub(super) map: HashMap<u64, T>,
    pub(super) dirty_map: HashMap<u64, T>,
    pub(super) dirty: bool,
}

impl<T> AssetMap<T> where AssetMap<T>: AssetMapTrait {
    pub(super) fn with_asset(first_key: u64, first_asset: T) -> AssetMap<T> {
        let mut dirty_map = HashMap::new();
        dirty_map.insert(first_key, first_asset);
        AssetMap {
            map: HashMap::new(),
            dirty_map,
            dirty: true,
        }
    }

}

pub trait AssetMapTrait: std::any::Any {
    fn dirty(&self) -> bool;
    fn reload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static + AssetTrait> AssetMapTrait for AssetMap<T> {
    fn dirty(&self) -> bool {
        self.dirty
    }
    
    fn reload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.dirty = false;
        for (key, mut value) in self.dirty_map.drain() {
            value.relaod(device, queue);
            self.map.insert(key, value);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any { self }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

