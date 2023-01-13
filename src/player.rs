use std::{fs::OpenOptions, io::Write, path::PathBuf};

use bevy::{
    prelude::{shape::Icosphere, *},
    render::view::NoFrustumCulling,
};
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    camera::FollowCamera,
    server::FrameState,
    world::{checkpoint::Checkpoint, terrain::Terrain},
    FrameStateSenderResource, HistoryResource, NextFrameResource, RuntimeResoure, SavePathReource,
    ShutdownResource,
};

pub struct PlayerPlugin {
    pub grpc: bool,
    pub recording_paths: Vec<PathBuf>,
    pub materials: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Resource)]
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

#[derive(Resource)]
pub struct PlayerSetupResource {
    pub paths: Vec<PathBuf>,
    pub materials: Vec<PathBuf>,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerMovement {
            transforms: Vec::new(),
        })
        .insert_resource(PlayerSetupResource {
            paths: self.recording_paths.clone(),
            materials: self.materials.clone(),
        })
        .add_startup_system(setup_ui)
        .add_system(kill_system)
        .add_system(record_player_positions)
        .add_system(sync_palyer_lights)
        .add_system(swap_camera)
        .add_system(player_light_system);
        if self.grpc && self.recording_paths.is_empty() {
            app.add_system(player_input_grpc)
                .add_system(send_player_view_grpc.before(player_input_grpc));
        } else if self.recording_paths.is_empty() {
            app.add_system(player_debug_inputs);
        } else {
            app.add_system(movement_playback);
        }
    }
}
#[derive(Component)]
pub struct CurrentPlayerText {}
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "PlayerName",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::TOP_CENTER)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        CurrentPlayerText {},
    ));
}

pub fn setup_player(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    paths: &[PathBuf],
    player_materials: &[PathBuf],
    start_block: (Vec3, f32),
) -> Vec<Entity> {
    let recordings = read_recordings(paths);
    if recordings.is_empty() {
        vec![spawn_player(
            commands,
            meshes,
            materials,
            start_block,
            (
                Vec::new(),
                StandardMaterial {
                    base_color: Color::ANTIQUE_WHITE,
                    ..Default::default()
                },
            ),
            0,
            "self".into(),
        )]
    } else {
        let player_materials = if player_materials.len() < recordings.len() {
            let mut m = player_materials
                .iter()
                .map(|p| StandardMaterial {
                    base_color_texture: Some(asset_server.load(p.as_path())),
                    ..Default::default()
                })
                .collect::<Vec<_>>();

            for _ in 0..(recordings.len() - player_materials.len()) {
                m.push(StandardMaterial {
                    base_color: Color::WHITE,
                    ..Default::default()
                });
            }
            m
        } else {
            player_materials
                .iter()
                .map(|p| StandardMaterial {
                    base_color_texture: Some(asset_server.load(p.as_path())),
                    ..Default::default()
                })
                .collect::<Vec<_>>()
        };
        recordings
            .into_iter()
            .zip(player_materials.into_iter())
            .enumerate()
            .map(|(i, ((player_name, transforms), player_material))| {
                spawn_player(
                    commands,
                    meshes,
                    materials,
                    start_block,
                    (transforms.transforms, player_material),
                    i,
                    player_name,
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
    player_info: (Vec<SerializableTransform>, StandardMaterial),
    index: usize,
    name: String,
) -> Entity {
    let playback_len = player_info.0.len();
    /* Create the bouncing ball. */
    let mut player_entity = commands.spawn(PbrBundle {
        mesh: meshes.add(
            Icosphere {
                radius: 0.5,
                subdivisions: 3,
            }
            .into(),
        ),
        material: materials.add(player_info.1),
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
        .insert(Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::ZERO,
        })
        .insert(PlayerMarker {
            playback_recording: player_info.0,
            playback_position: 0,
            index,
            name,
            current_position: None,
        });
    let player_entity = if playback_len == 0 {
        player_entity
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(0.5))
            .insert(Restitution::coefficient(1.0))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .id()
    } else {
        player_entity
            .insert(Sensor)
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(Collider::ball(0.5))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .id()
    };
    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::IDENTITY,
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
    pub current_position: Option<usize>,
}
#[derive(Component)]
struct PlayerLight {
    player: Entity,
}
fn player_debug_inputs(
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Velocity, With<PlayerMarker>>,
) {
    for mut impulse in player_query.iter_mut() {
        let x = 10.0 * keys.pressed(KeyCode::D) as i32 as f32
            + -10.0 * keys.pressed(KeyCode::A) as i32 as f32;
        let z = 10.0 * keys.pressed(KeyCode::S) as i32 as f32
            + -10.0 * keys.pressed(KeyCode::W) as i32 as f32;

        let y = 10.0 * keys.pressed(KeyCode::Space) as i32 as f32;
        impulse.linvel = Vec3::new(
            if x == 0.0 { impulse.linvel.x } else { x },
            if y == 0.0 { impulse.linvel.y } else { y },
            if z == 0.0 { impulse.linvel.z } else { z },
        );
    }
}

fn player_input_grpc(
    runtime: Res<RuntimeResoure>,
    mut next_frame_receiver: ResMut<NextFrameResource>,
    mut player_query: Query<&mut Velocity, With<PlayerMarker>>,
) {
    runtime.0.block_on(async {
        let force = next_frame_receiver.0.recv().await.unwrap();
        if force.x != 0.0 || force.z != 0.0 {
            for mut impulse in player_query.iter_mut() {
                impulse.linvel = Vec3::new(
                    force.x.clamp(-10.0, 10.0),
                    impulse.linvel.y,
                    force.z.clamp(-10.0, 10.0),
                );
            }
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
    runtime: Res<RuntimeResoure>,
    state_sender: Res<FrameStateSenderResource>,
    terrain: Res<Terrain>,
    player_query: Query<(Entity, &Transform, &Velocity), With<PlayerMarker>>,
    history: Res<HistoryResource>,
    checkpoints: Query<(&Checkpoint, &Transform)>,
) {
    let history = history.0.lock().unwrap();
    let next_state = if let Some((player, player_position, velocity)) = player_query.iter().next() {
        let history = history.get(&player).unwrap();
        let next_checkpoint_index = history
            .collected_checkpoints
            .last()
            .map(|c| c.0 + 1)
            .unwrap_or(0);
        let distance_to_next_checkpint = checkpoints
            .iter()
            .find(|c| c.0.number == next_checkpoint_index)
            .map(|c| c.1.translation.distance(player_position.translation))
            .unwrap_or(0.0);
        let next_checkpint = checkpoints
            .iter()
            .find(|c| c.0.number == next_checkpoint_index)
            .map(|c| c.1.translation);

        let surrounding = terrain
            .get_heights_around(player_position.translation.x, player_position.translation.z)
            .into_iter()
            .map(|q| q.map(|q| (q.texture, q.height)))
            .collect();

        Some(FrameState {
            surrounding,
            player: player_position.translation,
            distance: distance_to_next_checkpint,
            checkpoint: next_checkpint.unwrap_or(Vec3::ZERO),
            velocity: velocity.linvel,
            finished: next_checkpint.is_none(),
        })
    } else {
        None
    };
    drop(history);
    if let Some(next_state) = next_state {
        runtime.0.block_on(async {
            state_sender.0.send(next_state).await.unwrap();
        });
    }
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
    mut shutdown_receiver: ResMut<ShutdownResource>,
    positions: Res<PlayerMovement>,
    save_path: Res<SavePathReource>,
) {
    let receievd = shutdown_receiver.0.try_recv().is_ok();
    if keys.just_pressed(KeyCode::Escape) || receievd {
        if let Some(save_path) = &save_path.0 {
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

fn swap_camera(
    keys: Res<Input<KeyCode>>,
    mut players: Query<(&mut FollowCamera, &PlayerMarker)>,
    mut current_player_text_q: Query<&mut Text, With<CurrentPlayerText>>,
) {
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
    if let Some(player_name) = players.iter().find(|p| p.0.follows) {
        for mut t in current_player_text_q.iter_mut() {
            t.sections[0].value = player_name.1.name.clone();
        }
    }
}

fn player_light_system(
    players: Query<(Entity, &PlayerMarker)>,
    mut lights: Query<(&mut PointLight, &PlayerLight)>,
    history: Res<HistoryResource>,
) {
    let history = history.0.lock().unwrap();
    let mut sorted_players: Vec<_> = players
        .iter()
        .map(|(e, p)| {
            (
                e,
                p.current_position,
                history[&e].collected_checkpoints.len(),
            )
        })
        .collect();
    sorted_players.sort_unstable_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));
    for (position, (entity, _, _)) in sorted_players.iter().enumerate() {
        if let Some((mut light, _)) = lights.iter_mut().find(|l| l.1.player == *entity) {
            light.intensity = (position < 3) as u32 as f32 * 15000.0;
            light.color = match position {
                0 => Color::GOLD,
                1 => Color::SILVER,
                2 => Color::CRIMSON,
                _ => Color::WHITE,
            };
        }
    }
}
