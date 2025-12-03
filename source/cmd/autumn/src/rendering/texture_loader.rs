use bevy::prelude::*;

/// Load an image texture with nearest neighbor filtering and no mipmaps.
/// This ensures crisp, pixel-perfect rendering without any interpolation or blurring.
///
/// Settings applied:
/// - `mag_filter: Nearest` - No interpolation when scaling up
/// - `min_filter: Nearest` - No interpolation when scaling down
/// - `mipmap_filter: Nearest` - No interpolation between mipmap levels
/// - `lod_min_clamp: 0.0, lod_max_clamp: 0.0` - Disables mipmaps completely
/// - `address_mode: Repeat` - Suitable for tiling textures
pub fn load_image_texture<T: Into<String>>(
    asset_server: &Res<AssetServer>,
    path: T,
) -> Handle<Image> {
    asset_server.load_with_settings(
        path.into(),
        |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.is_srgb = true;
            settings.sampler =
                bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                    mag_filter: bevy::image::ImageFilterMode::Nearest,
                    min_filter: bevy::image::ImageFilterMode::Nearest,
                    mipmap_filter: bevy::image::ImageFilterMode::Nearest,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 0.0,
                    ..Default::default()
                });
        },
    )
}

/// Load a weapon texture with nearest neighbor filtering, no mipmaps, and clamp-to-edge addressing.
/// Similar to `load_image_texture` but uses `ClampToEdge` addressing mode which is better
/// for non-tiling sprites and UI elements.
pub fn load_weapon_texture<T: Into<String>>(
    asset_server: &Res<AssetServer>,
    path: T,
) -> Handle<Image> {
    asset_server.load_with_settings(
        path.into(),
        |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.is_srgb = true;
            settings.sampler =
                bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                    label: Some("weapon_sampler".into()),
                    address_mode_u: bevy::image::ImageAddressMode::ClampToEdge,
                    address_mode_v: bevy::image::ImageAddressMode::ClampToEdge,
                    address_mode_w: bevy::image::ImageAddressMode::ClampToEdge,
                    mag_filter: bevy::image::ImageFilterMode::Nearest,
                    min_filter: bevy::image::ImageFilterMode::Nearest,
                    mipmap_filter: bevy::image::ImageFilterMode::Nearest,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 0.0,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                });
        },
    )
}
