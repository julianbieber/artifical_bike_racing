use std::sync::{Arc, Mutex};

use bevy::math::Affine2;
use bevy::prelude::{shape::Icosphere, *};
use bevy::render::view::NoFrustumCulling;
use bevy_rapier3d::prelude::*;
use rand::distributions::Standard;
use rand::prelude::*;
use rand::rngs::SmallRng;

use crate::player::PlayerMarker;

use super::terrain::Terrain;

#[derive(Component)]
pub struct Checkpoint {
    number: u8,
}
pub struct History {
    pub total: i32,
    pub collected_checkpoints: Vec<(u8, usize)>,
}
impl History {
    fn next(&self) -> u8 {
        self.collected_checkpoints
            .last()
            .map(|l| l.0 + 1)
            .unwrap_or(0)
    }
}

pub fn setup_checkpoints(
    commands: &mut Commands,
    history: &Arc<Mutex<History>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    terrain: &mut Terrain,
    start_cube: Vec3,
) {
    let mut history = history.lock().unwrap();
    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.5, 0.0, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    let mesh = meshes.add(Mesh::from(Icosphere {
        radius: 3.0,
        subdivisions: 8,
    }));

    spawn_checkpoint(
        0,
        start_cube + Vec3::Y,
        commands,
        mesh.clone(),
        material.clone(),
    );
    let track = create_track(Vec2::new(start_cube.x, start_cube.z));
    let mut track_with_start = vec![Vec2::new(start_cube.x, start_cube.z)];
    track_with_start.extend(track.iter());
    terrain.register_road(&track_with_start);
    history.total = track_with_start.len() as i32;
    for (i, c) in track.into_iter().enumerate() {
        if let Some(height) = terrain.get_height(c.x, c.y) {
            spawn_checkpoint(
                (i + 1) as u8,
                Vec3::new(c.x, height + 3.0, c.y),
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
        .insert(NoFrustumCulling {})
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

pub struct FrameCounter {
    pub count: usize,
}

pub fn checkpoint_collection(
    mut commands: Commands,
    history: ResMut<Arc<Mutex<History>>>,
    mut frame_counter: ResMut<FrameCounter>,
    mut collision_events: EventReader<CollisionEvent>,
    checkpoints: Query<&Checkpoint>,
    player_query: Query<Entity, With<PlayerMarker>>,
) {
    if let Some(player) = player_query.iter().next() {
        frame_counter.count += 1;
        let mut history = history.lock().unwrap();
        for e in collision_events.iter() {
            match e {
                CollisionEvent::Started(e1, e2, _) if *e1 == player => {
                    if let Ok(checkpoint) = checkpoints.get(*e2) {
                        if checkpoint.number == history.next() {
                            commands.entity(*e2).despawn_recursive();
                            history
                                .collected_checkpoints
                                .push((checkpoint.number, frame_counter.count));
                        }
                    }
                }
                CollisionEvent::Started(e1, e2, _) if *e2 == player => {
                    if let Ok(checkpoint) = checkpoints.get(*e1) {
                        if checkpoint.number == history.next() {
                            commands.entity(*e1).despawn_recursive();
                            history
                                .collected_checkpoints
                                .push((checkpoint.number, frame_counter.count));
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
    history: Res<Arc<Mutex<History>>>,
) {
    let next = history.lock().unwrap().next();
    for (mut v, c) in checkpoints.iter_mut() {
        v.is_visible = c.number == next;
    }
}

fn create_track(start: Vec2) -> Vec<Vec2> {
    let generator = TrackGenerator::new(2, Vec2::new(-1.0, -10.0));
    generator.generate(start)
}

struct TrackGenerator {
    rng: SmallRng,
    current_direction: Vec2,
    state: DirectionState,
    same_direction_count: u32,
}
impl TrackGenerator {
    fn new(seed: u64, direction: Vec2) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
            current_direction: direction,
            state: DirectionState::Forward,
            same_direction_count: 1,
        }
    }

    fn generate(mut self, start: Vec2) -> Vec<Vec2> {
        let mut current = start;
        (0..50)
            .map(|_| {
                current += self.current_direction;
                self.step();
                current
            })
            .collect()
    }

    fn step(&mut self) {
        let random = self.rng.gen_range(0..10);
        let change = random < self.same_direction_count;
        self.same_direction_count = !change as u32 * self.same_direction_count + 1;
        if change {
            self.state = self.rng.gen::<DirectionState>();
        }
        self.current_direction = match self.state {
            DirectionState::Forward => self.current_direction,
            DirectionState::Left => {
                let direction = Affine2::from_translation(self.current_direction);
                (Affine2::from_angle(self.rng.gen_range(0.0..1.0)) * direction).translation
            }
            DirectionState::Right => {
                let direction = Affine2::from_translation(self.current_direction);
                (Affine2::from_angle(self.rng.gen_range(-1.0..0.0)) * direction).translation
            }
        }
    }
}

#[derive(Clone, Copy)]
enum DirectionState {
    Forward,
    Left,
    Right,
}

impl Distribution<DirectionState> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DirectionState {
        match rng.gen_range(0..=2) {
            0 => DirectionState::Forward,
            1 => DirectionState::Left,
            2 => DirectionState::Right,
            _ => unreachable!(),
        }
    }
}
