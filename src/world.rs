use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::texture::{create_texture, PbrImages};
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
    let mesh = generate_world(30, (-10.0, -10.0), 20.0);
    let mesh = meshes.add(mesh);
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

fn generate_world(subdivisions: usize, offset: (f32, f32), size: f32) -> Mesh {
    let vertices_per_length = subdivisions + 1;
    let mut vertices = Vec::with_capacity(vertices_per_length * vertices_per_length);
    let f_sub = subdivisions as f32;
    let step = size / f_sub;
    for z in 0..vertices_per_length {
        let f_z = z as f32 * step + offset.0;
        for x in 0..vertices_per_length {
            let f_x = x as f32 * step + offset.1;
            vertices.push(([f_x, 0.0, f_z], [0.0, 1.0, 0.0], [0.0, 0.0]));
        }
    }
    // let vertices = &[
    //     // Top
    //     ([sp.min_x, sp.min_y, sp.max_z], [0., 0., 1.0], [0., 0.]),
    //     ([sp.max_x, sp.min_y, sp.max_z], [0., 0., 1.0], [1.0, 0.]),
    //     ([sp.max_x, sp.max_y, sp.max_z], [0., 0., 1.0], [1.0, 1.0]),
    //     ([sp.min_x, sp.max_y, sp.max_z], [0., 0., 1.0], [0., 1.0]),
    // ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    dbg!(&positions);
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let mut indices: Vec<u32> =
        Vec::with_capacity((vertices_per_length) * (vertices_per_length) * 6);
    for x in 0..subdivisions as u32 {
        for y in 0..subdivisions as u32 {
            indices.extend_from_slice(&[
                x,
                x + (y + 1) * vertices_per_length as u32,
                x + 1,
                x + (y + 1) * vertices_per_length as u32,
                x + (y + 1) * vertices_per_length as u32 + 1,
                x + 1,
            ]);
        }
    }
    let indices = Indices::U32(dbg!(indices));

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}
