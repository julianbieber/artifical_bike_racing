pub mod checkpoint;
pub mod load_texture;
mod noise;
pub mod terrain;

use bevy::{
    prelude::{shape::Cube, *},
    render::view::NoFrustumCulling,
};

use bevy_rapier3d::prelude::*;

use crate::{
    player::{setup_player, RecordingPathsResource},
    HistoryResource,
};

use self::{
    checkpoint::{
        checkpoint_collection, only_show_next_checkpoint, setup_checkpoints, FrameCounter,
    },
    load_texture::setup_texture_atlas,
    terrain::Terrain,
};

pub struct WorldPlugin {}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameCounter { count: 0 })
            .add_system(checkpoint_collection)
            .add_system(only_show_next_checkpoint)
            .add_startup_system(setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    history: Res<HistoryResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    player_recordings: Res<RecordingPathsResource>,
) {
    let atlas = setup_texture_atlas(&mut images);
    let mut terrain = Terrain::new(430, 1.0);
    let start_cube_position =
        setup_start_cube(&mut commands, &terrain, &mut meshes, &mut materials);
    let players = setup_player(
        &mut commands,
        &mut meshes,
        &mut materials,
        &player_recordings.0,
        start_cube_position,
    );
    setup_checkpoints(
        &mut commands,
        &history.0,
        &mut meshes,
        &mut materials,
        &mut terrain,
        start_cube_position,
        &players,
    );
    let (mesh, collider) = terrain.to_mesh(&atlas);
    commands.insert_resource(terrain);
    let mesh = meshes.add(mesh);
    commands
        .spawn(PbrBundle {
            mesh,
            material: materials.add(atlas.material),
            ..Default::default()
        })
        .insert(NoFrustumCulling {})
        .insert(collider);
}

fn setup_start_cube(
    commands: &mut Commands,
    terrain: &Terrain,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> (Vec3, f32) {
    let (_min, max) = dbg!(terrain.get_dimensions());
    if let Some(edge_height) = terrain.get_height(0.0, dbg!(max.y - 1.0)) {
        let cube_position = Vec3::new(0.0, edge_height - 2.0, max.y);
        commands
            .spawn(PbrBundle {
                transform: Transform::from_translation(cube_position),
                mesh: meshes.add(Mesh::from(Cube::new(4.0))),
                material: materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert(RigidBody::Fixed)
            .insert(NoFrustumCulling {})
            .insert(Collider::cuboid(2.0, 2.0, 2.0));
        (cube_position, 4.0)
    } else {
        panic!("cound not palce start block");
    }
}
