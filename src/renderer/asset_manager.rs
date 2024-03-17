pub(crate) mod asset;
pub(crate) mod asset_map;

use std::collections::HashMap;

use self::{asset_map::{AssetMapTrait, AssetMap}, asset::AssetTrait};



/// Struct that can load and provide all kinds of assets.
pub(crate) struct AssetManager {
    /// Stores the type id to the type
    /// Safety: the entry at typeid T is always T
    assets: HashMap<std::any::TypeId, Box<dyn AssetMapTrait>>,
    dirty: bool,
}

impl AssetManager {
    pub(crate) fn new() -> AssetManager {
        AssetManager {
            assets: HashMap::new(),
            dirty: false,
        }
    }

    pub(crate) fn load<T: 'static + AssetTrait>(&mut self, key: u64, asset: T) -> Option<T>
        where AssetMap<T>: AssetMapTrait
    {
        self.dirty = true;
        match self.assets.get_mut(&std::any::TypeId::of::<AssetMap<T>>()) {
            Some(map) => {
                // SAFETY: safe because of our guarantee that the value at type id T is T
                let map = map.as_any_mut().downcast_mut::<AssetMap<T>>().unwrap();
                map.dirty = true;
                map.dirty_map.insert(key, asset)
            },
            None => {
                self.assets.insert(
                    std::any::TypeId::of::<AssetMap<T>>(),
                    Box::new(AssetMap::with_asset(key, asset))
                );
                None
            }
        }
    }

    pub(crate) fn get<T: 'static + AssetTrait>(&self, key: u64) -> Option<&T>
        where AssetMap<T>: AssetMapTrait
    {
        let map = self.assets.get(&std::any::TypeId::of::<AssetMap<T>>())?;
        // SAFETY: safe because of our guarantee that the value at type id T is T
        let map = map.as_any().downcast_ref::<AssetMap<T>>().unwrap();
        map.map.get(&key)
    }

    
    pub(crate) fn dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn reload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.dirty = false;

        for asset_map in self.assets.values_mut() {
            if asset_map.dirty() {
                asset_map.reload(device, queue)
            }
        }
    }
}