use bevy::prelude::{shape::Icosphere, *};
use bevy_rapier3d::prelude::*;

use crate::camera::FollowCamera;

pub struct PlayerPlugin {}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_system(player_debug_inputs);
    }
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the bouncing ball. */
    commands
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
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
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
        .insert(PlayerMarker {});
}

#[derive(Component)]
struct PlayerMarker {}

fn player_debug_inputs(
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<&mut ExternalForce, With<PlayerMarker>>,
) {
    for mut impulse in player_query.iter_mut() {
        impulse.force = Vec3::Y * 10.0 * keys.pressed(KeyCode::Space) as i32 as f32;
    }
}
