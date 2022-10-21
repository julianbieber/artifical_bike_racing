use bevy::{
    prelude::*,
    window::{PresentMode, WindowMode},
};
use bevy_rapier3d::prelude::*;
use camera::CameraPlugin;
use clap::Parser;
use player::PlayerPlugin;
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

mod camera;
mod player;
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
    .add_plugin(CameraPlugin {})
    .add_plugin(PlayerPlugin {})
    .add_system(kill_system);
    if !opt.cont {
        a.add_system(print_ball_altitude);
    }

    a.run();
    t.join().unwrap();
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
