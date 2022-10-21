mod load_texture;
mod noise;
mod terrain;

use crate::world::load_texture::TextureSections;
use bevy::prelude::{shape::Cube, *};

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
    let (mesh, collider, terrain) = generate_world(&atlas, 430, 1.0);
    setup_start_cube(&mut commands, &terrain, &mut meshes, &mut materials);
    commands.insert_resource(terrain);
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

fn setup_start_cube(
    commands: &mut Commands,
    terrain: &Terrain,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let (_min, max) = dbg!(terrain.get_dimensions());
    if let Some(edge_height) = terrain.get_height(0.0, dbg!(max.y - 1.0)) {
        let cube_position = Vec3::new(0.0, edge_height - 2.0, max.y);
        commands
            .spawn_bundle(PbrBundle {
                transform: Transform::from_translation(cube_position),
                mesh: meshes.add(Mesh::from(Cube::new(4.0))),
                material: materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert(StartBlock { size: 4.0 })
            .insert(RigidBody::Fixed)
            .insert(Collider::cuboid(2.0, 2.0, 2.0));
    } else {
        panic!("cound not palce start block");
    }
}

#[derive(Component)]
pub struct StartBlock {
    pub size: f32,
}

fn generate_world(
    atlas: &Atlas<TextureSections>,
    subdivisions: usize,
    size: f32,
) -> (Mesh, Collider, Terrain) {
    let quads = Terrain::new(subdivisions, size);
    let (mesh, collider) = quads.to_mesh(atlas);
    (mesh, collider, quads)
}
