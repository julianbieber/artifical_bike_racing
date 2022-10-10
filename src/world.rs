use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::{
    noise::WorldNoise,
    texture::{create_texture, Atlas, PbrImages},
};
use bevy_rapier3d::prelude::*;
use statrs::statistics::Statistics;

pub struct WorldPlugin {}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let atlas = create_texture(
        &[(
            // https://ambientcg.com/view?id=Grass004
            TextureSections::Grass,
            PbrImages {
                color: "assets/grass/color.png".into(),
                normal: "assets/grass/normal.png".into(),
                roughness: "assets/grass/roughness.png".into(),
                ambient: "assets/grass/ambient.png".into(),
            },
        )],
        &mut images,
    );
    let (mesh, collider) = generate_world(&atlas, 430, 1.0);
    let mesh = meshes.add(mesh);
    commands
        .spawn_bundle(PbrBundle {
            mesh,
            material: materials.add(atlas.material),
            ..Default::default()
        })
        .insert(collider);
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
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(4.5))
        .insert(Restitution::coefficient(0.9));
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                color: Color::ALICE_BLUE,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 22.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(4.5))
        .insert(Restitution::coefficient(1.9));
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                color: Color::CRIMSON,
                ..default()
            },
            transform: Transform::from_xyz(-4.0, 8.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(2.5))
        .insert(Restitution::coefficient(0.8));
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                color: Color::GOLD,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(2.5))
        .insert(Restitution::coefficient(0.7));
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum TextureSections {
    Grass,
    Grass2,
}

fn generate_world(
    atlas: &Atlas<TextureSections>,
    subdivisions: usize,
    size: f32,
) -> (Mesh, Collider) {
    let quads = WorldQuads::new(subdivisions, size);
    quads.to_mesh(atlas)
}

struct WorldQuads {
    quads: Vec<Vec<Quad>>,
    size: f32,
}

impl WorldQuads {
    fn new(size: usize, s: f32) -> WorldQuads {
        let noise = WorldNoise::new();
        let quads = (0..size)
            .map(|x| {
                (0..size)
                    .map(|z| {
                        let height = noise.get_height(x, z);
                        Quad {
                            height,
                            texture: TextureSections::Grass,
                            scale: s,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        WorldQuads {
            quads,
            size: size as f32,
        }
    }

    fn to_mesh(&self, atlas: &Atlas<TextureSections>) -> (Mesh, Collider) {
        let mut positions: Vec<[f32; 3]> =
            Vec::with_capacity(self.quads.len() * self.quads.len() * 4);
        let mut normals: Vec<[f32; 3]> =
            Vec::with_capacity(self.quads.len() * self.quads.len() * 4);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(self.quads.len() * self.quads.len() * 4);
        let mut indices = Vec::with_capacity(self.quads.len() * self.quads.len() * 6);
        let mut current_index = 0;

        for (x, quads) in self.quads.iter().enumerate() {
            for (z, quad) in quads.iter().enumerate() {
                let s = surrounding_indices(x, z)
                    .map(|row| row.map(|(x1, z1)| self.get(x1, z1).map(|q| q.height)));
                positions.extend(quad.to_positions(
                    x as f32 - self.size / 2.0,
                    z as f32 - self.size / 2.0,
                    &s,
                ));
                normals.extend(quad.to_normals());
                uvs.extend(quad.to_uvs(atlas));
                indices.extend([
                    current_index,
                    current_index + 2,
                    current_index + 1,
                    current_index + 2,
                    current_index + 3,
                    current_index + 1,
                ]);
                current_index += 4;
            }
        }

        let collider = Collider::trimesh(
            positions
                .iter()
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .collect(),
            indices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect(),
        );
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        (mesh, collider)
    }

    fn get(&self, x: usize, z: usize) -> Option<&Quad> {
        self.quads.get(x).and_then(|q| q.get(z))
    }
}
fn surrounding_indices(x: usize, z: usize) -> [[(usize, usize); 3]; 3] {
    [
        [(x - 1, z - 1), (x, z - 1), (x + 1, z - 1)],
        [(x - 1, z), (x, z), (x + 1, z)],
        [(x - 1, z + 1), (x, z + 1), (x + 1, z + 1)],
    ]
}

struct Quad {
    height: f32,
    texture: TextureSections,
    scale: f32,
}
impl Quad {
    fn to_positions(&self, x: f32, z: f32, surrounding: &[[Option<f32>; 3]; 3]) -> [[f32; 3]; 4] {
        [
            [
                self.scale * (x + 0.0),
                [
                    surrounding[0][0],
                    surrounding[0][1],
                    surrounding[1][0],
                    surrounding[1][1],
                ]
                .into_iter()
                .map(|v| v.unwrap_or(self.height) as f64)
                .mean() as f32,
                self.scale * (z + 0.0),
            ],
            [
                self.scale * (x + 1.0),
                [
                    surrounding[0][1],
                    surrounding[0][2],
                    surrounding[1][1],
                    surrounding[1][2],
                ]
                .into_iter()
                .map(|v| v.unwrap_or(self.height) as f64)
                .mean() as f32,
                self.scale * (z + 0.0),
            ],
            [
                self.scale * (x + 0.0),
                [
                    surrounding[1][0],
                    surrounding[1][1],
                    surrounding[2][0],
                    surrounding[2][1],
                ]
                .into_iter()
                .map(|v| v.unwrap_or(self.height) as f64)
                .mean() as f32,
                self.scale * (z + 1.0),
            ],
            [
                self.scale * (x + 1.0),
                [
                    surrounding[1][1],
                    surrounding[1][2],
                    surrounding[2][1],
                    surrounding[2][2],
                ]
                .into_iter()
                .map(|v| v.unwrap_or(self.height) as f64)
                .mean() as f32,
                self.scale * (z + 1.0),
            ],
        ]
    }

    fn to_normals(&self) -> [[f32; 3]; 4] {
        [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn to_uvs(&self, atlas: &Atlas<TextureSections>) -> [[f32; 2]; 4] {
        let coords = atlas.to_uv.get(&self.texture).unwrap();
        [
            [coords.left, coords.bottom],
            [coords.right, coords.bottom],
            [coords.left, coords.top],
            [coords.right, coords.top],
        ]
    }
}
