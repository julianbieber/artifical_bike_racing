use crate::{texture::Atlas, world::noise::WorldNoise};

use super::load_texture::TextureSections;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_rapier3d::prelude::Collider;
use statrs::statistics::Statistics;

pub struct Terrain {
    quads: Vec<Vec<Quad>>,
    size: f32,
}

impl Terrain {
    pub fn new(size: usize, s: f32) -> Terrain {
        let mut min_height = std::f32::INFINITY;
        let mut max_height = std::f32::NEG_INFINITY;
        let noise = WorldNoise::new();
        let quads = (0..size)
            .map(|x| {
                (0..size)
                    .map(|z| {
                        let height = noise.get_height(x, z);
                        min_height = min_height.min(height);
                        max_height = max_height.max(height);
                        Quad {
                            height,
                            texture: to_texture(height),
                            scale: s,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        dbg!(min_height, max_height);
        Terrain {
            quads,
            size: size as f32,
        }
    }

    pub fn to_mesh(&self, atlas: &Atlas<TextureSections>) -> (Mesh, Collider) {
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
                let (world_x, world_z) = self.index_to_world(x, z);
                let (p, n) = quad.to_positions_and_normals(world_x, world_z, &s);
                positions.extend(p);
                normals.extend(n);
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

    pub fn get_height(&self, x: f32, z: f32) -> Option<f32> {
        let (x, z) = self.world_to_index(x, z);
        self.get(x, z).map(|q| q.height)
    }

    fn get(&self, x: usize, z: usize) -> Option<&Quad> {
        self.quads.get(x).and_then(|q| q.get(z))
    }

    fn index_to_world(&self, x: usize, z: usize) -> (f32, f32) {
        (x as f32 - self.size / 2.0, z as f32 - self.size / 2.0)
    }

    fn world_to_index(&self, x: f32, z: f32) -> (usize, usize) {
        (
            (x + self.size * 2.0) as usize,
            (z + self.size * 2.0) as usize,
        )
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
    fn to_positions_and_normals(
        &self,
        x: f32,
        z: f32,
        surrounding: &[[Option<f32>; 3]; 3],
    ) -> ([[f32; 3]; 4], [[f32; 3]; 4]) {
        let positions = [
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
        ];
        let n0 = normal(positions[0], positions[2], positions[1]).to_array();
        let n1 = ((normal(positions[1], positions[0], positions[2])
            + normal(positions[1], positions[2], positions[3]))
            / 2.0)
            .to_array();
        let n2 = ((normal(positions[1], positions[0], positions[2])
            + normal(positions[1], positions[2], positions[3]))
            / 2.0)
            .to_array();
        let n3 = normal(positions[3], positions[1], positions[2]).to_array();
        let normals = [n0, n1, n2, n3];

        (positions, normals)
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

// normal at a, with a triangle spanned by a_b and a_c
fn normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Vec3 {
    let a = Vec3::from_array(a);
    let b = Vec3::from_array(b);
    let c = Vec3::from_array(c);
    let v = b - a;
    let w = c - a;
    Vec3::new(
        v.y * w.z - v.z * w.y,
        v.z * w.x - v.x * w.z,
        v.x * w.y - v.y * w.x,
    )
    .normalize_or_zero()
}

fn to_texture(height: f32) -> TextureSections {
    match height {
        x if x < -5.0 => TextureSections::Grass,
        x if x < 0.0 => TextureSections::Grass2,
        x if x < 5.0 => TextureSections::Gravel,
        x if x < 7.0 => TextureSections::Rock,
        _ => TextureSections::Snow,
    }
}
