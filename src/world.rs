use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use noise::{NoiseFn, Simplex};

use crate::texture::{create_texture, Atlas, PbrImages};
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
    let mesh = generate_world(&atlas, 30, (-10.0, -10.0), 20.0);
    let mesh = meshes.add(mesh);
    commands.spawn_bundle(PbrBundle {
        mesh,
        material: materials.add(atlas.material),
        ..Default::default()
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum TextureSections {
    Grass,
}

fn generate_world(
    atlas: &Atlas<TextureSections>,
    subdivisions: usize,
    offset: (f32, f32),
    size: f32,
) -> Mesh {
    let quads = WorldQuads::new(subdivisions);
    quads.to_mesh(atlas)
}

struct WorldQuads {
    quads: Vec<Vec<Quad>>,
}

impl WorldQuads {
    fn new(size: usize) -> WorldQuads {
        let noise = Simplex::new(0);
        let quads = (0..size)
            .map(|x| {
                (0..size)
                    .map(|z| {
                        let height = noise.get([scale(x), scale(z)]) as f32;
                        Quad {
                            height,
                            texture: TextureSections::Grass,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        WorldQuads { quads }
    }

    fn to_mesh(&self, atlas: &Atlas<TextureSections>) -> Mesh {
        let mut positions: Vec<[f32; 3]> =
            Vec::with_capacity(self.quads.len() * self.quads.len() * 4);
        let mut normals: Vec<[f32; 3]> =
            Vec::with_capacity(self.quads.len() * self.quads.len() * 4);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(self.quads.len() * self.quads.len() * 4);
        let mut indices = Vec::with_capacity(self.quads.len() * self.quads.len() * 6);
        let mut current_index = 0;

        for (x, quads) in self.quads.iter().enumerate() {
            for (z, quad) in quads.iter().enumerate() {
                let x_p1 = self.get(x + 1, z).map(|q| q.height);
                let x_m1 = self.get(x - 1, z).map(|q| q.height);
                let z_p1 = self.get(x, z + 1).map(|q| q.height);
                let z_m1 = self.get(x, z - 1).map(|q| q.height);
                positions.extend(quad.to_positions(
                    x as f32 - 10.0,
                    z as f32 - 10.0,
                    x_p1,
                    x_m1,
                    z_p1,
                    z_m1,
                ));
                normals.extend(quad.to_normals());
                uvs.extend(quad.to_uvs(atlas));
                indices.extend([
                    current_index,
                    current_index + 2,
                    current_index + 1,
                    current_index + 1,
                    current_index + 2,
                    current_index + 3,
                ]);
                current_index += 4;
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh
    }

    fn get(&self, x: usize, z: usize) -> Option<&Quad> {
        self.quads.get(x).and_then(|q| q.get(z))
    }
}

fn scale(v: usize) -> f64 {
    (v as f64) / 1000.0
}

struct Quad {
    height: f32,
    texture: TextureSections,
}
impl Quad {
    fn to_positions(
        &self,
        x: f32,
        z: f32,
        height_x_p1: Option<f32>,
        height_x_m1: Option<f32>,
        height_y_p1: Option<f32>,
        height_y_m1: Option<f32>,
    ) -> [[f32; 3]; 4] {
        [
            [x + 0.0, 0.0, z + 0.0],
            [x + 1.0, 0.0, z + 1.0],
            [x + 0.0, 0.0, z + 0.0],
            [x + 1.0, 0.0, z + 1.0],
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
        [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]]
    }
}
