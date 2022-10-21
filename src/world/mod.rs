mod load_texture;
mod noise;
mod terrain;

use crate::world::load_texture::TextureSections;
use bevy::prelude::*;

use crate::texture::Atlas;
use bevy_rapier3d::prelude::*;

use self::{load_texture::setup_texture_atlas, terrain::Terrain};

pub struct WorldPlugin {}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let atlas = setup_texture_atlas(&mut images);
    let (mesh, collider) = generate_world(&atlas, 430, 1.0);
    let mesh = meshes.add(mesh);
    commands
        .spawn_bundle(PbrBundle {
            mesh,
            material: materials.add(atlas.material),
            ..Default::default()
        })
        .insert(collider);
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                radius: 100.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 22.0, 50.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(4.5))
        .insert(Restitution::coefficient(0.9));
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                color: Color::ALICE_BLUE,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 22.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(4.5))
        .insert(Restitution::coefficient(1.9));
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                color: Color::CRIMSON,
                ..default()
            },
            transform: Transform::from_xyz(-4.0, 8.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(2.5))
        .insert(Restitution::coefficient(0.8));
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                color: Color::GOLD,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(2.5))
        .insert(Restitution::coefficient(0.7));
}

fn generate_world(
    atlas: &Atlas<TextureSections>,
    subdivisions: usize,
    size: f32,
) -> (Mesh, Collider) {
    let quads = Terrain::new(subdivisions, size);
    quads.to_mesh(atlas)
}
