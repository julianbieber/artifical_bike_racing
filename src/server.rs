use std::thread::JoinHandle;

use tokio::{
    runtime::Runtime,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};
use tonic::{transport::Server, Request, Response, Status};

use self::game::main_service_server::MainServiceServer;
use self::game::{main_service_server::MainService, Empty, InputRequest};

pub mod game {
    #![allow(clippy::all)]
    tonic::include_proto!("game");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("game_descriptor");
}

#[derive(Debug)]
pub struct FrameState {}
#[derive(Debug)]
pub struct NextFrame {
    pub x: f32,
    pub z: f32,
}

pub fn start_server(
    frame_receiver: Receiver<FrameState>,
    next_frame_sender: Sender<NextFrame>,
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
}

#[tonic::async_trait]
impl MainService for GameServer {
    async fn health(&self, _r: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
    async fn get_state(&self, _r: Request<Empty>) -> Result<Response<Empty>, Status> {
        let mut receievr = self.frame_receiver.lock().await;
        if let Some(_state) = receievr.recv().await {
            Ok(Response::new(Empty {}))
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
        std::process::exit(0);
    }
}
