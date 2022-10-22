use bevy::prelude::{shape::Icosphere, *};
use bevy_rapier3d::prelude::*;

use crate::player::PlayerMarker;

use super::terrain::Terrain;

#[derive(Component)]
pub struct Checkpoint {
    number: u8,
}
pub struct History {
    pub collected_checkpoints: Vec<u8>,
}

pub fn setup_checkpoints(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    _terrain: &Terrain,
    start_cube: Vec3,
) {
    commands
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(start_cube),
            mesh: meshes.add(Mesh::from(Icosphere {
                radius: 3.0,
                subdivisions: 8,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.5, 0.0, 0.5),
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(Collider::ball(3.0))
        .insert(Sensor)
        .insert(Checkpoint { number: 0 });
}

pub fn checkpoint_collection(
    mut commands: Commands,
    mut history: ResMut<History>,
    mut collision_events: EventReader<CollisionEvent>,
    checkpoints: Query<&Checkpoint>,
    player_query: Query<Entity, With<PlayerMarker>>,
) {
    if let Some(player) = player_query.iter().next() {
        for e in collision_events.iter() {
            match e {
                CollisionEvent::Started(e1, e2, _) if *e1 == player => {
                    if let Ok(checkpoint) = checkpoints.get(*e2) {
                        if checkpoint.number
                            == history
                                .collected_checkpoints
                                .last()
                                .map(|l| l + 1)
                                .unwrap_or(0)
                        {
                            commands.entity(*e2).despawn_recursive();
                            history.collected_checkpoints.push(checkpoint.number);
                        }
                    }
                }
                CollisionEvent::Started(e1, e2, _) if *e2 == player => {
                    if let Ok(checkpoint) = checkpoints.get(*e1) {
                        if checkpoint.number
                            == history
                                .collected_checkpoints
                                .last()
                                .map(|l| l + 1)
                                .unwrap_or(0)
                        {
                            commands.entity(*e1).despawn_recursive();
                            history.collected_checkpoints.push(checkpoint.number);
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
