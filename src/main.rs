use std::sync::{Arc, Mutex};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowMode},
};
use bevy_rapier3d::prelude::*;
use camera::CameraPlugin;
use clap::Parser;
use player::PlayerPlugin;
use server::start_server;
use tokio::runtime::Runtime;
use world::{checkpoint::History, WorldPlugin};

mod camera;
mod player;
mod server;
mod texture;
mod world;

#[derive(Parser, Copy, Clone, Debug)]
struct Opt {
    #[arg(long)]
    port: i32,
    #[arg(long)]
    cont: bool,
}

fn main() {
    let opt = dbg!(Opt::parse());
    let runtime = Runtime::new().unwrap();
    let (frame_sender, frame_reciever) = tokio::sync::mpsc::channel(1);
    let (next_sender, next_reciever) = tokio::sync::mpsc::channel(1);

    let history = Arc::new(Mutex::new(History {
        collected_checkpoints: Vec::with_capacity(256),
        total: 0,
    }));
    let t = start_server(frame_reciever, next_sender, history.clone(), opt.port);
    let mut a = App::new();
    if opt.cont {
        a.insert_resource(WindowDescriptor {
            mode: WindowMode::Fullscreen,
            present_mode: PresentMode::AutoVsync,
            ..default()
        });
    }
    a.insert_resource(next_reciever)
        .insert_resource(history)
        .insert_resource(frame_sender)
        .insert_resource(runtime)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(WorldPlugin {})
        .add_plugin(CameraPlugin { active: opt.cont })
        .add_plugin(PlayerPlugin { grpc: !opt.cont })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin {
            ..Default::default()
        })
        .add_system(kill_system);

    a.run();
    t.join().unwrap();
}

fn kill_system(keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}
