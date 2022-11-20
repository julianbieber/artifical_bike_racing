use std::{fs::OpenOptions, io::Write, path::PathBuf};

use bevy::{
    prelude::{shape::Icosphere, *},
    render::view::NoFrustumCulling,
};
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    camera::FollowCamera,
    server::{FrameState, NextFrame},
    world::terrain::Terrain,
};

pub struct PlayerPlugin {
    pub grpc: bool,
    pub recording_paths: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct PlayerMovement {
    transforms: Vec<SerializableTransform>,
}
#[derive(Serialize, Deserialize)]
struct SerializableTransform {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

impl From<&Transform> for SerializableTransform {
    fn from(t: &Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerMovement {
            transforms: Vec::new(),
        })
        .insert_resource(self.recording_paths.clone())
        .add_system(kill_system)
        .add_system(record_player_positions)
        .add_system(sync_palyer_lights)
        .add_system(swap_camera);
        if self.grpc {
            app.add_system(player_input_grpc)
                .add_system(send_player_view_grpc);
        } else if self.recording_paths.is_empty() {
            app.add_system(player_debug_inputs);
        } else {
            app.add_system(movement_playback);
        }
    }
}

pub fn setup_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    paths: &[PathBuf],
    start_block: (Vec3, f32),
) -> Vec<Entity> {
    let recordings = read_recordings(paths);
    if recordings.is_empty() {
        vec![spawn_player(
            commands,
            meshes,
            materials,
            start_block,
            Vec::new(),
            0,
            "self".into(),
        )]
    } else {
        recordings
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                spawn_player(
                    commands,
                    meshes,
                    materials,
                    start_block,
                    r.1.transforms,
                    i,
                    r.0,
                )
            })
            .collect()
    }
}

fn spawn_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    start_block_transform: (Vec3, f32),
    playback: Vec<SerializableTransform>,
    index: usize,
    name: String,
) -> Entity {
    let playback_len = playback.len();
    /* Create the bouncing ball. */
    let mut player_entity = commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(
            Icosphere {
                radius: 0.5,
                subdivisions: 5,
            }
            .into(),
        ),
        material: materials.add(StandardMaterial {
            ..Default::default()
        }),
        transform: Transform::from_translation(
            start_block_transform.0 + Vec3::Y * start_block_transform.1,
        ),
        ..Default::default()
    });
    player_entity
        .insert(NoFrustumCulling {})
        .insert(FollowCamera {
            follows: index == 0,
        })
        .insert(ExternalForce {
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
        })
        .insert(PlayerMarker {
            playback_recording: playback,
            playback_position: 0,
            index,
            name,
        });
    let player_entity = if playback_len == 0 {
        player_entity
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(0.5))
            .insert(Restitution::coefficient(0.7))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .id()
    } else {
        player_entity
            .insert(Sensor)
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(0.5))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .id()
    };
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
        .insert(PlayerLight {
            player: player_entity,
        });
    player_entity
}

#[derive(Component)]
pub struct PlayerMarker {
    pub name: String,
    playback_recording: Vec<SerializableTransform>,
    playback_position: usize,
    index: usize,
}
#[derive(Component)]
struct PlayerLight {
    player: Entity,
}
fn player_debug_inputs(
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<&mut ExternalForce, With<PlayerMarker>>,
) {
    for mut impulse in player_query.iter_mut() {
        impulse.force = Vec3::Y * 10.0 * keys.pressed(KeyCode::Space) as i32 as f32
            + Vec3::Z * 10.0 * keys.pressed(KeyCode::W) as i32 as f32
            + Vec3::Z * -10.0 * keys.pressed(KeyCode::S) as i32 as f32
            + Vec3::X * 10.0 * keys.pressed(KeyCode::A) as i32 as f32
            + Vec3::X * -10.0 * keys.pressed(KeyCode::D) as i32 as f32;
    }
}

fn player_input_grpc(
    runtime: Res<Runtime>,
    mut next_frame_receiver: ResMut<Receiver<NextFrame>>,
    mut player_query: Query<&mut ExternalForce, With<PlayerMarker>>,
) {
    runtime.block_on(async {
        let force = next_frame_receiver.recv().await.unwrap();
        for mut impulse in player_query.iter_mut() {
            impulse.force = Vec3::new(force.x, 0.0, force.z);
        }
    });
}

fn sync_palyer_lights(
    player_transforms: Query<&Transform, Without<PlayerLight>>,
    mut lights: Query<(&mut Transform, &PlayerLight)>,
) {
    for (mut light_transform, player) in lights.iter_mut() {
        if let Ok(player_transform) = player_transforms.get(player.player) {
            light_transform.translation = player_transform.translation + Vec3::Y * 10.0;
        }
    }
}

fn send_player_view_grpc(
    runtime: Res<Runtime>,
    state_sender: Res<Sender<FrameState>>,
    terrain: Res<Terrain>,
    player_query: Query<&Transform, With<PlayerMarker>>,
) {
    runtime.block_on(async {
        if let Some(player_position) = player_query.iter().next() {
            let surrounding = terrain
                .get_heights_around(player_position.translation.x, player_position.translation.z)
                .into_iter()
                .map(|q| q.map(|q| (q.texture, q.height)))
                .collect();
            state_sender
                .send(FrameState {
                    surrounding,
                    player: player_position.translation,
                })
                .await
                .unwrap();
        }
    });
}

fn record_player_positions(
    mut positions: ResMut<PlayerMovement>,
    player_query: Query<&Transform, With<PlayerMarker>>,
) {
    if let Some(p) = player_query.iter().next() {
        positions.transforms.push(p.into());
    }
}

fn kill_system(
    keys: Res<Input<KeyCode>>,
    mut shutdown_receiver: ResMut<Receiver<()>>,
    positions: Res<PlayerMovement>,
    save_path: Res<Option<PathBuf>>,
) {
    let receievd = shutdown_receiver.try_recv().is_ok();
    if keys.just_pressed(KeyCode::Escape) || receievd {
        if let Some(save_path) = save_path.into_inner() {
            let positions_json = serde_json::to_string(positions.into_inner()).unwrap();
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .truncate(true)
                .open(save_path)
                .unwrap();
            file.write_all(positions_json.as_bytes()).unwrap();
        };
        std::process::exit(0);
    }
}

fn read_recordings(paths: &[PathBuf]) -> Vec<(String, PlayerMovement)> {
    paths
        .iter()
        .map(|path| {
            let j = std::fs::read_to_string(path).unwrap();
            (
                path.file_name().unwrap().to_string_lossy().into(),
                serde_json::from_str(&j).unwrap(),
            )
        })
        .collect()
}

fn movement_playback(mut players_q: Query<(&mut Transform, &mut PlayerMarker)>) {
    for (mut t, mut p) in players_q.iter_mut() {
        if let Some(next) = p
            .playback_recording
            .get(p.playback_position)
            .or_else(|| p.playback_recording.last())
        {
            t.translation = next.translation.into();
            t.rotation = Quat::from_array(next.rotation);
            t.scale = next.scale.into();
            p.playback_position += 1;
        }
    }
}

fn swap_camera(keys: Res<Input<KeyCode>>, mut players: Query<(&mut FollowCamera, &PlayerMarker)>) {
    if keys.just_pressed(KeyCode::Right) {
        let player_count = players.iter().count();
        if let Some(current_follow) = players.iter().find(|p| p.0.follows) {
            let next = (current_follow.1.index + 1) % player_count;
            players
                .iter_mut()
                .for_each(|mut p| p.0.follows = p.1.index == next);
        }
    }
    if keys.just_pressed(KeyCode::Left) {
        let player_count = players.iter().count();
        if let Some(current_follow) = players.iter().find(|p| p.0.follows) {
            let next = (current_follow.1.index - 1) % player_count;
            players
                .iter_mut()
                .for_each(|mut p| p.0.follows = p.1.index == next);
        }
    }
}
