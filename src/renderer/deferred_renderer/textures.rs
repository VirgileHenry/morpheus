
// all the textures that are used by the deferred renderer.

use super::texture::TextureTypeInfo;

pub(super) struct AlbedoTexture;
impl TextureTypeInfo for AlbedoTexture {
    #[cfg(debug_assertions)]
    const LABEL: &'static str = "albedo";
}

pub(super) struct NormalDepthTexture;
impl TextureTypeInfo for NormalDepthTexture {
    #[cfg(debug_assertions)]
    const LABEL: &'static str = "normal and depth";
}