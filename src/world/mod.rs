pub mod checkpoint;
pub mod load_texture;
mod noise;
pub mod terrain;

use bevy::{
    prelude::{shape::Icosphere, *},
    render::view::NoFrustumCulling,
};

use bevy_rapier3d::prelude::*;

use crate::{
    player::{setup_player, PlayerSetupResource},
    HistoryResource,
};

use self::{
    checkpoint::{
        build_checkpoints, checkpoint_collection, only_show_next_checkpoint, Checkpoint,
        FrameCounter, History,
    },
    load_texture::setup_texture_atlas,
    terrain::Terrain,
};

pub struct WorldPlugin {
    pub seed: u32,
}
#[derive(Resource)]
struct Seed {
    value: u32,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameCounter { count: 0 })
            .insert_resource(Seed { value: self.seed })
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
    player_recordings: Res<PlayerSetupResource>,
    seed: Res<Seed>,
) {
    let mut history = history.0.lock().unwrap();
    let atlas = setup_texture_atlas(&mut images);
    let mut terrain = Terrain::new(430, 1.0, seed.value);
    let checkpoints = build_checkpoints(&mut materials, &mut terrain, seed.value);
    let players = setup_player(
        &mut commands,
        &mut meshes,
        &mut materials,
        &player_recordings.paths,
        &player_recordings.colors,
        (checkpoints[0].0, 2.0),
    );
    let checkpoint_mesh = meshes.add(
        Icosphere {
            radius: 3.0,
            subdivisions: 8,
        }
        .into(),
    );

    *history = players
        .iter()
        .map(|e| {
            (
                *e,
                History {
                    total: checkpoints.len() as i32,
                    collected_checkpoints: Vec::with_capacity(255),
                },
            )
        })
        .collect();
    for (trnaslation, mut checkpoint) in checkpoints {
        checkpoint.remaining_players = players.clone();
        checkpoint.total_player_count = players.len();
        spawn_checkpoint(
            &mut commands,
            trnaslation,
            checkpoint,
            checkpoint_mesh.clone(),
        );
    }
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

fn spawn_checkpoint(
    commands: &mut Commands,
    translation: Vec3,
    checkpoint: Checkpoint,
    checkpoint_mesh: Handle<Mesh>,
) {
    commands
        .spawn(PbrBundle {
            transform: Transform::from_translation(translation),
            mesh: checkpoint_mesh,
            material: checkpoint.first_place_color.clone(),
            ..Default::default()
        })
        .insert(NoFrustumCulling {})
        .insert(Collider::ball(3.0))
        .insert(Sensor)
        .insert(checkpoint);
}
