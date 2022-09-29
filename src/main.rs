use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use clap::Parser;
use server::{start_server, NextFrame};
use tokio::{runtime::Runtime, sync::mpsc::Receiver};
#[derive(Parser, Copy, Clone)]
struct Opt {
    #[arg(long)]
    port: i32,
}

mod server;

fn main() {
    let opt = Opt::parse();
    let runtime = Runtime::new().unwrap();
    let (frame_sender, frame_reciever) = tokio::sync::mpsc::channel(1);
    let (next_sender, next_reciever) = tokio::sync::mpsc::channel(1);

    let t = start_server(frame_reciever, next_sender, opt.port);
    App::new()
        .insert_resource(next_reciever)
        .insert_resource(frame_sender)
        .insert_resource(runtime)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(print_ball_altitude)
        .run();
    t.join().unwrap();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    /* Create the bouncing ball. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)));
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
