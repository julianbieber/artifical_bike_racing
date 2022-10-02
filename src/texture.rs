use std::path::PathBuf;

use bevy::{
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat},
    utils::HashMap,
};
use image::{ImageBuffer, RgbaImage};

pub struct Atlas<A> {
    material: StandardMaterial,
    to_uv: HashMap<A, (f32, f32)>,
}

pub struct PbrImages {
    color: PathBuf,
    normal: PathBuf,
    roughness: PathBuf,
    ambient: PathBuf,
}

pub fn create_texture<A>(images: &[(A, PbrImages)], image_server: &mut Assets<Image>) -> Atlas<A> {
    let color: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let normal: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let roughness: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let ambient: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let uvs = HashMap::with_capacity(images.len());
    for (i, image) in images.iter().enumerate() {}

    Atlas {
        material: StandardMaterial {
            base_color_texture: Some(image_server.add(to_bevy_image(color))),
            ..Default::default()
        },
        to_uv: uvs,
    }
}

fn to_bevy_image(image: RgbaImage) -> Image {
    Image::new(
        bevy::render::render_resource::Extent3d {
            width: image.dimensions().0,
            height: image.dimensions().1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image.as_raw().to_vec(),
        TextureFormat::Rgba8Unorm,
    )
}
