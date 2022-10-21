use bevy::prelude::*;

use crate::texture::{create_texture, Atlas, PbrImages};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureSections {
    Grass,
    Grass2,
    Gravel,
    Rock,
    Snow,
}

pub fn setup_texture_atlas(images: &mut Assets<Image>) -> Atlas<TextureSections> {
    create_texture(
        &[
            (
                // https://ambientcg.com/view?id=Grass004
                TextureSections::Grass,
                PbrImages {
                    color: "assets/grass/color.png".into(),
                    normal: "assets/grass/normal.png".into(),
                    roughness: "assets/grass/roughness.png".into(),
                    ambient: Some("assets/grass/ambient.png".into()),
                },
            ),
            (
                // https://ambientcg.com/view?id=Grass004
                TextureSections::Grass2,
                PbrImages {
                    color: "assets/grass2/color.png".into(),
                    normal: "assets/grass2/normal.png".into(),
                    roughness: "assets/grass2/roughness.png".into(),
                    ambient: Some("assets/grass2/ambient.png".into()),
                },
            ),
            (
                // https://ambientcg.com/view?id=Grass004
                TextureSections::Gravel,
                PbrImages {
                    color: "assets/gravel/color.png".into(),
                    normal: "assets/gravel/normal.png".into(),
                    roughness: "assets/gravel/roughness.png".into(),
                    ambient: Some("assets/gravel/ambient.png".into()),
                },
            ),
            (
                // https://ambientcg.com/view?id=Grass004
                TextureSections::Rock,
                PbrImages {
                    color: "assets/rock/color.png".into(),
                    normal: "assets/rock/normal.png".into(),
                    roughness: "assets/rock/roughness.png".into(),
                    ambient: Some("assets/rock/ambient.png".into()),
                },
            ),
            (
                // https://ambientcg.com/view?id=Grass004
                TextureSections::Snow,
                PbrImages {
                    color: "assets/snow/color.png".into(),
                    normal: "assets/snow/normal.png".into(),
                    roughness: "assets/snow/roughness.png".into(),
                    ambient: None,
                },
            ),
        ],
        images,
    )
}
