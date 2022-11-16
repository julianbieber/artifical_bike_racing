use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::math::Affine2;
use bevy::prelude::{shape::Icosphere, *};
use bevy::render::view::NoFrustumCulling;
use bevy::utils::HashSet;
use bevy_rapier3d::prelude::*;
use rand::distributions::Standard;
use rand::prelude::*;
use rand::rngs::SmallRng;

use crate::player::PlayerMarker;

use super::terrain::Terrain;

#[derive(Component)]
pub struct Checkpoint {
    number: u8,
    remaining_players: Vec<Entity>,
    total_player_count: usize,
    first_place_color: Handle<StandardMaterial>,
    remaining_color: Handle<StandardMaterial>,
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
    fn finished(&self) -> bool {
        self.collected_checkpoints.len() as i32 == self.total
    }
}

pub fn setup_checkpoints(
    commands: &mut Commands,
    history: &Arc<Mutex<HashMap<Entity, History>>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    terrain: &mut Terrain,
    start_cube: (Vec3, f32),
    players: &Vec<Entity>,
) {
    let mut history = history.lock().unwrap();
    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.5, 0.0, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    let material_2 = materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.5, 0.0, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
    let mesh = meshes.add(Mesh::from(Icosphere {
        radius: 3.0,
        subdivisions: 8,
    }));

    spawn_checkpoint(
        0,
        start_cube.0 + Vec3::Y * start_cube.1,
        commands,
        mesh.clone(),
        material.clone(),
        material_2.clone(),
        players,
    );
    let track = create_track(Vec2::new(start_cube.0.x, start_cube.0.z));
    let mut track_with_start = vec![Vec2::new(start_cube.0.x, start_cube.0.z)];
    track_with_start.extend(track.iter());
    terrain.register_road(&track_with_start);
    for (i, c) in track.into_iter().enumerate() {
        if let Some(height) = terrain.get_height(c.x, c.y) {
            spawn_checkpoint(
                (i + 1) as u8,
                Vec3::new(c.x, height + 3.0, c.y),
                commands,
                mesh.clone(),
                material.clone(),
                material_2.clone(),
                players,
            );
        }
    }
    *history = players
        .iter()
        .map(|e| {
            (
                *e,
                History {
                    total: track_with_start.len() as i32,
                    collected_checkpoints: Vec::with_capacity(255),
                },
            )
        })
        .collect();
}

fn spawn_checkpoint(
    number: u8,
    translation: Vec3,
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    material_2: Handle<StandardMaterial>,
    players: &Vec<Entity>,
) {
    commands
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(translation),
            mesh,
            material: material.clone(),
            ..Default::default()
        })
        .insert(NoFrustumCulling {})
        .insert(Collider::ball(3.0))
        .insert(Sensor)
        .insert(Checkpoint {
            number,
            remaining_players: players.clone(),
            total_player_count: players.len(),
            first_place_color: material,
            remaining_color: material_2,
        });
    // .with_children(|cb| {
    //     cb.spawn_bundle(PointLightBundle {
    //         point_light: PointLight {
    //             intensity: 15000.0,
    //             radius: 5.0,
    //             shadows_enabled: true,
    //             color: Color::AQUAMARINE,
    //             ..default()
    //         },
    //         ..Default::default()
    //     });
    // });
}

pub struct FrameCounter {
    pub count: usize,
}
/// TODO send collection events instead of writing into the history
/// frame counter per player
pub fn checkpoint_collection(
    mut commands: Commands,
    history: ResMut<Arc<Mutex<HashMap<Entity, History>>>>,
    mut frame_counter: ResMut<FrameCounter>,
    mut collision_events: EventReader<CollisionEvent>,
    mut checkpoints: Query<(Entity, &mut Checkpoint)>,
    player_query: Query<(Entity, &PlayerMarker)>,
) {
    let collision_events: Vec<CollisionEvent> = collision_events.iter().cloned().collect();
    let players: HashSet<Entity> = player_query.iter().map(|v| v.0).collect();
    if players.is_empty() {
        frame_counter.count += 1;
        let mut history = history.lock().unwrap();
        for e in collision_events.iter() {
            match e {
                CollisionEvent::Started(e1, e2, _) if players.contains(e1) => {
                    collect_cp(
                        *e1,
                        *e2,
                        &mut history,
                        &mut checkpoints,
                        frame_counter.count,
                        &player_query,
                    );
                }
                CollisionEvent::Started(e1, e2, _) if players.contains(e2) => {
                    collect_cp(
                        *e2,
                        *e1,
                        &mut history,
                        &mut checkpoints,
                        frame_counter.count,
                        &player_query,
                    );
                }
                _ => (),
            }
        }
    }
    for (e, c) in checkpoints.iter() {
        if c.remaining_players.is_empty() {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn collect_cp(
    player_entity: Entity,
    cp_entity: Entity,
    history: &mut HashMap<Entity, History>,
    checkpoints: &mut Query<(Entity, &mut Checkpoint)>,
    frame_counter: usize,
    player_query: &Query<(Entity, &PlayerMarker)>,
) {
    if let Ok((_, mut checkpoint)) = checkpoints.get_mut(cp_entity) {
        let history = history.get_mut(&player_entity).unwrap();
        if checkpoint.number == history.next() {
            checkpoint.remaining_players.retain(|e| *e != cp_entity);
            history
                .collected_checkpoints
                .push((checkpoint.number, frame_counter));
            if history.finished() {
                let player = player_query.get(player_entity).unwrap().1;
                dbg!(
                    &player.name,
                    "finished the track after",
                    Duration::from_millis(frame_counter as u64 * 16)
                );
            }
        }
    }
}

pub fn only_show_next_checkpoint(
    mut checkpoints: Query<(&mut Visibility, &mut Handle<StandardMaterial>, &Checkpoint)>,
    history: Res<Arc<Mutex<HashMap<Entity, History>>>>,
) {
    let max_next_cp = history
        .lock()
        .unwrap()
        .iter()
        .map(|(_, h)| h.next())
        .max()
        .unwrap();
    for (mut v, mut t, c) in checkpoints.iter_mut() {
        v.is_visible = c.number == max_next_cp || c.remaining_players.len() != c.total_player_count;
        *t = if c.remaining_players.len() == c.total_player_count {
            c.first_place_color.clone()
        } else {
            c.remaining_color.clone()
        }
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
