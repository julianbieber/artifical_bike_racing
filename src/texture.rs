use std::{
    cmp::Eq,
    hash::Hash,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat},
    utils::HashMap,
};
use image::{imageops::FilterType, ImageBuffer, RgbaImage};

pub struct Atlas<A> {
    pub material: StandardMaterial,
    pub to_uv: HashMap<A, UvCoords>,
}

pub struct UvCoords {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

pub struct PbrImages {
    pub color: PathBuf,
    pub normal: PathBuf,
    pub roughness: PathBuf,
    pub ambient: Option<PathBuf>,
}

pub fn create_texture<A>(images: &[(A, PbrImages)], image_server: &mut Assets<Image>) -> Atlas<A>
where
    A: Eq + Hash + Copy + Clone,
{
    let mut color: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let mut normal: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let mut roughness: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let mut ambient: RgbaImage = ImageBuffer::new(1024 * images.len() as u32, 1024);
    let mut uvs = HashMap::with_capacity(images.len());
    for (i, (marker, pbr_image)) in images.iter().enumerate() {
        let current = read_image(&pbr_image.color);
        set_section(&current, &mut color, i as u32 * 1024);
        let current = read_image(&pbr_image.normal);
        set_section(&current, &mut normal, i as u32 * 1024);
        let current = read_image(&pbr_image.roughness);
        set_section(&current, &mut roughness, i as u32 * 1024);
        let current = pbr_image
            .ambient
            .as_ref()
            .map(|p| read_image(&p))
            .unwrap_or_else(|| black_image());
        set_section(&current, &mut ambient, i as u32 * 1024);

        uvs.insert(
            *marker,
            UvCoords {
                top: 1.0,
                bottom: 0.0,
                left: i as f32 / (images.len() as f32),
                right: (i + 1) as f32 / (images.len() as f32),
            },
        );
    }

    Atlas {
        material: StandardMaterial {
            base_color_texture: Some(image_server.add(to_bevy_image(color))),
            normal_map_texture: Some(image_server.add(to_bevy_image(normal))),
            metallic_roughness_texture: Some(image_server.add(to_bevy_image(roughness))),
            occlusion_texture: Some(image_server.add(to_bevy_image(ambient))),
            ..Default::default()
        },
        to_uv: uvs,
    }
}

fn read_image(path: &Path) -> RgbaImage {
    image::io::Reader::open(path)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .resize_exact(1024, 1024, FilterType::Gaussian)
        .into_rgba8()
}

fn black_image() -> RgbaImage {
    let mut i = RgbaImage::new(1024, 1024);
    for x in 0..1024 {
        for y in 0..1024 {
            i.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        }
    }
    i
}

fn set_section(src: &RgbaImage, dst: &mut RgbaImage, width_offset: u32) {
    for x in 0..1024 {
        for y in 0..1024 {
            dst.put_pixel(x + width_offset, y, *src.get_pixel(x, y));
        }
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
