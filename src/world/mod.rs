mod checkpoint;
mod load_texture;
mod noise;
mod terrain;

use bevy::{
    prelude::{shape::Cube, *},
    render::view::NoFrustumCulling,
};

use bevy_rapier3d::prelude::*;

use self::{
    checkpoint::{checkpoint_collection, only_show_next_checkpoint, setup_checkpoints, History},
    load_texture::setup_texture_atlas,
    terrain::Terrain,
};

pub struct WorldPlugin {}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(History {
            collected_checkpoints: Vec::with_capacity(256),
        })
        .add_system(checkpoint_collection)
        .add_system(only_show_next_checkpoint)
        .add_startup_system(setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let atlas = setup_texture_atlas(&mut images);
    let mut terrain = Terrain::new(430, 1.0);
    let start_cube_position =
        setup_start_cube(&mut commands, &terrain, &mut meshes, &mut materials);
    setup_checkpoints(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut terrain,
        start_cube_position,
    );
    let (mesh, collider) = terrain.to_mesh(&atlas);
    commands.insert_resource(terrain);
    let mesh = meshes.add(mesh);
    commands
        .spawn_bundle(PbrBundle {
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
) -> Vec3 {
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
            .insert(NoFrustumCulling {})
            .insert(Collider::cuboid(2.0, 2.0, 2.0));
        cube_position
    } else {
        panic!("cound not palce start block");
    }
}

#[derive(Component)]
pub struct StartBlock {
    pub size: f32,
}
