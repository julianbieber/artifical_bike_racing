use std::collections::HashMap;
use std::time::Duration;

use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_rapier3d::prelude::*;
use rand::distributions::Standard;
use rand::prelude::*;
use rand::rngs::SmallRng;

use crate::player::PlayerMarker;
use crate::HistoryResource;

use super::terrain::Terrain;

#[derive(Component)]
pub struct Checkpoint {
    pub number: u8,
    pub remaining_players: Vec<Entity>,
    pub total_player_count: usize,
    pub first_place_color: Handle<StandardMaterial>,
    pub remaining_color: Handle<StandardMaterial>,
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

pub fn build_checkpoints(
    materials: &mut Assets<StandardMaterial>,
    terrain: &mut Terrain,
    seed: u32,
) -> Vec<(Vec3, Checkpoint)> {
    let start = {
        let x = 0.0;
        let z = terrain.get_dimensions().1.y / 2.0 - 1.0;
        let y = terrain.get_height(x, z).unwrap() + 1.0;
        Vec3::new(x, y, z)
    };
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

    let mut checkpoints = Vec::new();
    let track = create_track(Vec2::new(start.x, start.z), seed);
    let mut track_with_start = vec![Vec2::new(start.x, start.z)];
    track_with_start.extend(track.iter());
    terrain.register_road(&track_with_start);
    for (i, c) in track_with_start.into_iter().enumerate() {
        if let Some(height) = terrain.get_height(c.x, c.y) {
            checkpoints.push((
                Vec3::new(c.x, height + 3.0, c.y),
                build_checkpoint((i) as u8, material.clone(), material_2.clone()),
            ));
        } else {
            break;
        }
    }
    checkpoints
}

fn build_checkpoint(
    number: u8,
    material: Handle<StandardMaterial>,
    material_2: Handle<StandardMaterial>,
) -> Checkpoint {
    Checkpoint {
        number,
        remaining_players: Vec::new(),
        total_player_count: 0,
        first_place_color: material,
        remaining_color: material_2,
    }
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

#[derive(Resource)]
pub struct FrameCounter {
    pub count: usize,
}
/// TODO send collection events instead of writing into the history
/// frame counter per player
pub fn checkpoint_collection(
    mut commands: Commands,
    history: ResMut<HistoryResource>,
    mut frame_counter: ResMut<FrameCounter>,
    mut collision_events: EventReader<CollisionEvent>,
    mut checkpoints: Query<(Entity, &mut Checkpoint)>,
    mut player_query: Query<(Entity, &mut PlayerMarker)>,
) {
    let collision_events: Vec<CollisionEvent> = collision_events.iter().cloned().collect();
    let players: HashSet<Entity> = player_query.iter().map(|v| v.0).collect();
    if !players.is_empty() {
        frame_counter.count += 1;
        let mut history = history.0.lock().unwrap();
        for e in collision_events.iter() {
            match e {
                CollisionEvent::Started(e1, e2, _) if players.contains(e1) => {
                    collect_cp(
                        *e1,
                        *e2,
                        &mut history,
                        &mut checkpoints,
                        frame_counter.count,
                        &mut player_query,
                    );
                }
                CollisionEvent::Started(e1, e2, _) if players.contains(e2) => {
                    collect_cp(
                        *e2,
                        *e1,
                        &mut history,
                        &mut checkpoints,
                        frame_counter.count,
                        &mut player_query,
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
    histories: &mut HashMap<Entity, History>,
    checkpoints: &mut Query<(Entity, &mut Checkpoint)>,
    frame_counter: usize,
    player_query: &mut Query<(Entity, &mut PlayerMarker)>,
) {
    if let Ok((_, mut checkpoint)) = checkpoints.get_mut(cp_entity) {
        let number_of_players = histories.len();
        let history = histories.get_mut(&player_entity).unwrap();
        if checkpoint.number == history.next() {
            checkpoint.remaining_players.retain(|e| *e != player_entity);
            history
                .collected_checkpoints
                .push((checkpoint.number, frame_counter));

            let mut player = player_query.get_mut(player_entity).unwrap().1;
            player.current_position = Some(number_of_players - checkpoint.remaining_players.len());

            if history.finished() {
                dbg!(
                    &player.name,
                    "finished the track after",
                    Duration::from_millis(frame_counter as u64 * 16),
                    "on position",
                    player.current_position
                );
            }
        }
    }
}

pub fn only_show_next_checkpoint(
    mut checkpoints: Query<(&mut Visibility, &mut Handle<StandardMaterial>, &Checkpoint)>,
    history: Res<HistoryResource>,
) {
    let max_next_cp = history
        .0
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

fn create_track(start: Vec2, seed: u32) -> Vec<Vec2> {
    let generator = TrackGenerator::new(seed as u64, Vec2::new(-1.0, -10.0));
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
