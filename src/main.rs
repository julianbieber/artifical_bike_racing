use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bevy::{
    audio::AudioPlugin,
    core_pipeline::CorePipelinePlugin,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    gltf::GltfPlugin,
    pbr::PbrPlugin,
    prelude::*,
    render::RenderPlugin,
    sprite::SpritePlugin,
    text::TextPlugin,
    ui::UiPlugin,
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
    /// port used to start the grpc server
    #[arg(long)]
    port: i32,
    /// if passed, the game does not wait for grpc input. The simulation runs continously.
    #[arg(long)]
    continuous: bool,
    /// if passed, the game will not be rendered.
    #[arg(long)]
    headless: bool,
    /// The seed for world and track generation
    #[arg(long)]
    seed: u32,
    #[arg(long)]
    /// path to a previously recorded race. The file contains one player transformation (position + rotation) per frame of the previous run.
    /// The recording will be replayed without additional physics simulation.
    recording: Vec<PathBuf>,
    #[arg(long)]
    /// color for the recorded sphere. This parameter must be passed the same number of times as recording.
    color: Vec<PlayerColor>,
    #[arg(long)]
    /// Path under which to save a recoding.
    save: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum PlayerColor {
    Red,
    Green,
    Black,
    White,
    Yellow,
    Blue,
    Grey,
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
    if opt.color.len() != opt.recording.len() {
        panic!("color and recording must have the same length");
    }
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
        .insert_resource(SavePathReource(opt.save));
    if opt.headless {
        a.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    add_primary_window: false,
                    exit_on_all_closed: false,
                    ..Default::default()
                })
                .build()
                .disable::<AudioPlugin>()
                .disable::<RenderPlugin>()
                .disable::<PbrPlugin>()
                .disable::<SpritePlugin>()
                .disable::<TextPlugin>()
                .disable::<UiPlugin>()
                .disable::<GltfPlugin>()
                .disable::<AnimationPlugin>()
                .disable::<CorePipelinePlugin>()
                .disable::<GilrsPlugin>(),
        )
        .add_asset::<Mesh>()
        .add_asset::<StandardMaterial>();
    } else {
        a.add_plugins(DefaultPlugins).add_plugin(CameraPlugin {
            active: opt.continuous,
        });
    }
    a.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(configure_physics)
        .add_plugin(WorldPlugin { seed: opt.seed })
        .add_plugin(PlayerPlugin {
            grpc: !opt.continuous,
            recording_paths: opt.recording,
            colors: opt.color.into_iter().map(|v| v.into()).collect(),
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin {
            ..Default::default()
        });

    a.run();
    t.join().unwrap();
}

impl From<PlayerColor> for Color {
    fn from(c: PlayerColor) -> Self {
        match c {
            PlayerColor::Red => Color::RED,
            PlayerColor::Green => Color::GREEN,
            PlayerColor::Black => Color::BLACK,
            PlayerColor::White => Color::WHITE,
            PlayerColor::Yellow => Color::YELLOW,
            PlayerColor::Blue => Color::BLUE,
            PlayerColor::Grey => Color::GRAY,
        }
    }
}

fn configure_physics(mut config: ResMut<RapierConfiguration>) {
    config.timestep_mode = TimestepMode::Fixed {
        dt: 0.016,
        substeps: 1,
    };
}
