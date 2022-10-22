use bevy::prelude::{shape::Icosphere, *};
use bevy_rapier3d::prelude::*;

use crate::{camera::FollowCamera, world::StartBlock};

pub struct PlayerPlugin {}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Initialized { is: false })
            .add_system(setup_player)
            .add_system(player_debug_inputs)
            .add_system(sync_palyer_lights);
    }
}

struct Initialized {
    is: bool,
}

fn setup_player(
    mut initialized: ResMut<Initialized>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    start_block_query: Query<(&Transform, &StartBlock)>,
) {
    if !initialized.is {
        if let Some((start_block_transform, start_block)) = start_block_query.iter().next() {
            initialized.is = true;
            /* Create the bouncing ball. */
            let player_entity = commands
                .spawn_bundle(PbrBundle {
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
                        start_block_transform.translation + Vec3::Y * start_block.size,
                    ),
                    ..Default::default()
                })
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(0.5))
                .insert(Restitution::coefficient(0.7))
                .insert(FollowCamera { follows: true })
                .insert(ExternalForce {
                    force: Vec3::ZERO,
                    torque: Vec3::ZERO,
                })
                .insert(PlayerMarker {})
                .insert(ActiveEvents::COLLISION_EVENTS)
                .id();
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
        }
    }
}

#[derive(Component)]
pub struct PlayerMarker {}
#[derive(Component)]
struct PlayerLight {
    player: Entity,
}
fn player_debug_inputs(
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<&mut ExternalForce, With<PlayerMarker>>,
) {
    for mut impulse in player_query.iter_mut() {
        impulse.force = Vec3::Y * 10.0 * keys.pressed(KeyCode::Space) as i32 as f32;
    }
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
