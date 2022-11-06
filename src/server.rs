use bevy::prelude::Vec3;
use std::sync::Arc;
use std::thread::JoinHandle;
use tokio::{
    runtime::Runtime,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};
use tonic::{transport::Server, Request, Response, Status};

use crate::world::{checkpoint::History, load_texture::TextureSections};

use self::game::main_service_server::MainServiceServer;
use self::game::{
    main_service_server::MainService, Empty, InputRequest, PlayerView, Score, Terrain,
};

pub mod game {
    #![allow(clippy::all)]
    tonic::include_proto!("game");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("game_descriptor");
}

#[derive(Debug)]
pub struct FrameState {
    pub surrounding: Vec<Option<(TextureSections, f32)>>,
    pub player: Vec3,
}
#[derive(Debug)]
pub struct NextFrame {
    pub x: f32,
    pub z: f32,
}

pub fn start_server(
    frame_receiver: Receiver<FrameState>,
    next_frame_sender: Sender<NextFrame>,
    shutdown_sender: Sender<()>,
    history: Arc<std::sync::Mutex<History>>,
    port: i32,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            next_frame_sender
                .send(NextFrame { x: 0.0, z: 0.0 })
                .await
                .unwrap();
            let addr = format!("[::1]:{port}").parse().unwrap();
            let game_server = GameServer {
                frame_receiver: Mutex::new(frame_receiver),
                next_frame_sender,
                history,
                shutdown_sender,
            };
            println!("started server");
            let reflection = tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(game::FILE_DESCRIPTOR_SET)
                .build()
                .unwrap();
            Server::builder()
                .add_service(reflection)
                .add_service(MainServiceServer::new(game_server))
                .serve(addr)
                .await
                .unwrap();
        })
    })
}

/// Server has receiver for frame states
/// Server has sender for calculate_next events
pub struct GameServer {
    pub frame_receiver: Mutex<Receiver<FrameState>>,
    pub next_frame_sender: Sender<NextFrame>,
    pub shutdown_sender: Sender<()>,
    pub history: Arc<std::sync::Mutex<History>>,
}

#[tonic::async_trait]
impl MainService for GameServer {
    async fn health(&self, _r: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
    async fn get_state(&self, _r: Request<Empty>) -> Result<Response<PlayerView>, Status> {
        let mut receievr = self.frame_receiver.lock().await;
        if let Some(state) = receievr.recv().await {
            Ok(Response::new(PlayerView {
                surrounding: state
                    .surrounding
                    .iter()
                    .map(|p| {
                        p.map(|p| Terrain {
                            height: p.1,
                            kind: p.0 as i32,
                        })
                        .unwrap_or(Terrain {
                            height: 0.0,
                            kind: -1,
                        })
                    })
                    .collect(),
                y: state.player.y,
            }))
        } else {
            Err(Status::not_found("no new game state available"))
        }
    }
    async fn input(&self, r: Request<InputRequest>) -> Result<Response<Empty>, Status> {
        let input = r.into_inner();
        self.next_frame_sender
            .send(NextFrame {
                x: input.x,
                z: input.z,
            })
            .await
            .map_err(|e| Status::unknown(format!("{e:?}")))?;
        Ok(Response::new(Empty {}))
    }
    async fn kill(&self, _r: Request<Empty>) -> Result<Response<Empty>, Status> {
        let _ = self.shutdown_sender.send(()).await;
        Ok(Response::new(Empty {}))
    }
    async fn get_score(&self, _r: Request<Empty>) -> Result<Response<Score>, Status> {
        let status = {
            let history = self.history.lock().unwrap();
            Score {
                timings: history
                    .collected_checkpoints
                    .iter()
                    .map(|h| h.1 as i64)
                    .collect(),
                total: history.total,
            }
        };
        let _ = self.shutdown_sender.send(()).await;
        Ok(Response::new(status))
    }
}
