use bevy::{
    prelude::{shape::Icosphere, *},
    window::{PresentMode, WindowMode},
};
use bevy_rapier3d::prelude::*;
use clap::Parser;
use server::{start_server, NextFrame};
use tokio::{runtime::Runtime, sync::mpsc::Receiver};
use world::WorldPlugin;
#[derive(Parser, Copy, Clone, Debug)]
struct Opt {
    #[arg(long)]
    port: i32,
    #[arg(long)]
    cont: bool,
}

mod noise;
mod server;
mod texture;
mod world;

fn main() {
    let opt = dbg!(Opt::parse());
    let runtime = Runtime::new().unwrap();
    let (frame_sender, frame_reciever) = tokio::sync::mpsc::channel(1);
    let (next_sender, next_reciever) = tokio::sync::mpsc::channel(1);

    let t = start_server(frame_reciever, next_sender, opt.port);
    let mut a = App::new();
    a.insert_resource(WindowDescriptor {
        mode: WindowMode::Fullscreen,
        present_mode: PresentMode::AutoVsync,
        ..default()
    })
    .insert_resource(next_reciever)
    .insert_resource(frame_sender)
    .insert_resource(runtime)
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    // .add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(WorldPlugin {})
    .add_startup_system(setup_graphics)
    .add_startup_system(setup_physics)
    .add_system(kill_system);
    if !opt.cont {
        a.add_system(print_ball_altitude);
    }

    a.run();
    t.join().unwrap();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 20.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_physics(
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
        .insert(Restitution::coefficient(0.7));
}

fn print_ball_altitude(
    positions: Query<&Transform, With<RigidBody>>,
    runtime: Res<Runtime>,
    mut next_frame_receiver: ResMut<Receiver<NextFrame>>,
) {
    runtime.block_on(async {
        next_frame_receiver.recv().await.unwrap();
    });
    for transform in positions.iter() {
        println!("Ball altitude: {}", transform.translation.y);
    }
}

fn kill_system(keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}
