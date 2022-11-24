use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use camera::CameraPlugin;
use clap::Parser;
use player::PlayerPlugin;
use server::{start_server, FrameState, NextFrame};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
};
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
    #[arg(long)]
    save: Option<PathBuf>,
}

#[derive(Resource)]
pub struct RuntimeResoure(pub Runtime);
#[derive(Resource)]
pub struct FrameStateSenderResource(pub Sender<FrameState>);
#[derive(Resource)]
pub struct HistoryResource(pub Arc<Mutex<HashMap<Entity, History>>>);
#[derive(Resource)]
pub struct NextFrameResource(pub Receiver<NextFrame>);

#[derive(Resource)]
pub struct ShutdownResource(pub Receiver<()>);

#[derive(Resource)]
pub struct SavePathReource(pub Option<PathBuf>);

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
    a.insert_resource(NextFrameResource(next_reciever))
        .insert_resource(HistoryResource(history))
        .insert_resource(FrameStateSenderResource(frame_sender))
        .insert_resource(RuntimeResoure(runtime))
        .insert_resource(ShutdownResource(shutdown_receiver))
        .insert_resource(SavePathReource(opt.save))
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
