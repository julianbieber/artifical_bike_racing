use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

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

#[derive(Parser, Clone, Debug)]
struct Opt {
    #[arg(long)]
    port: i32,
    #[arg(long)]
    cont: bool,
    #[arg(long)]
    /// singular to make the cli more intuitive
    recording: Vec<PathBuf>,
}

fn main() {
    let opt = dbg!(Opt::parse());
    let runtime = Runtime::new().unwrap();
    let (frame_sender, frame_reciever) = tokio::sync::mpsc::channel(1);
    let (next_sender, next_reciever) = tokio::sync::mpsc::channel(1);
    let (shutdown_sender, shutdown_receiver) = tokio::sync::mpsc::channel(1);

    let history = Arc::new(Mutex::new(HashMap::<Entity, History>::with_capacity(
        opt.recording.len() + 1,
    )));
    let t = start_server(
        frame_reciever,
        next_sender,
        shutdown_sender,
        history.clone(),
        opt.port,
    );
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
        .insert_resource(shutdown_receiver)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(WorldPlugin {})
        .add_plugin(CameraPlugin { active: opt.cont })
        .add_plugin(PlayerPlugin {
            grpc: !opt.cont,
            recording_paths: opt.recording,
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin {
            ..Default::default()
        });

    a.run();
    t.join().unwrap();
}
