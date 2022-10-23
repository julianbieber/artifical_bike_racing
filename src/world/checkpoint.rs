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
impl History {
    fn next(&self) -> u8 {
        self.collected_checkpoints
            .last()
            .map(|l| l + 1)
            .unwrap_or(0)
    }
}

pub fn setup_checkpoints(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    _terrain: &Terrain,
    start_cube: Vec3,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.5, 0.0, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    let mesh = meshes.add(Mesh::from(Icosphere {
        radius: 3.0,
        subdivisions: 8,
    }));

    spawn_checkpoint(0, start_cube, commands, mesh.clone(), material.clone());
    for (i, c) in create_track(Vec2::new(start_cube.x, start_cube.z))
        .into_iter()
        .enumerate()
    {
        if let Some(height) = _terrain.get_height(c.x, c.y) {
            spawn_checkpoint(
                (i + 1) as u8,
                Vec3::new(c.x, height, c.y),
                commands,
                mesh.clone(),
                material.clone(),
            );
        }
    }
}

fn spawn_checkpoint(
    number: u8,
    translation: Vec3,
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    commands
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(translation),
            mesh,
            material,
            ..Default::default()
        })
        .insert(Collider::ball(3.0))
        .insert(Sensor)
        .insert(Checkpoint { number })
        .with_children(|cb| {
            cb.spawn_bundle(PointLightBundle {
                point_light: PointLight {
                    intensity: 15000.0,
                    radius: 5.0,
                    shadows_enabled: true,
                    color: Color::AQUAMARINE,
                    ..default()
                },
                ..Default::default()
            });
        });
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
                        if checkpoint.number == history.next() {
                            commands.entity(*e2).despawn_recursive();
                            history.collected_checkpoints.push(checkpoint.number);
                        }
                    }
                }
                CollisionEvent::Started(e1, e2, _) if *e2 == player => {
                    if let Ok(checkpoint) = checkpoints.get(*e1) {
                        if checkpoint.number == history.next() {
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

pub fn only_show_next_checkpoint(
    mut checkpoints: Query<(&mut Visibility, &Checkpoint)>,
    history: Res<History>,
) {
    let next = history.next();
    for (mut v, c) in checkpoints.iter_mut() {
        v.is_visible = c.number == next;
    }
}

fn create_track(start: Vec2) -> Vec<Vec2> {
    vec![start + Vec2::new(-1.0, -10.0)]
}
