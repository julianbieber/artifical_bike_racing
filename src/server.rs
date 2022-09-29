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
use self::game::{main_service_server::MainService, Empty};
pub mod game {
    tonic::include_proto!("game");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("game_descriptor");
}

pub struct FrameState {}
#[derive(Debug)]
pub struct NextFrame {}

pub fn start_server(
    frame_receiver: Receiver<FrameState>,
    next_frame_sender: Sender<NextFrame>,
) -> JoinHandle<()> {
    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let addr = "[::1]:50051".parse().unwrap();
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
    async fn health(&self, r: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
    async fn get_state(&self, r: Request<Empty>) -> Result<Response<Empty>, Status> {
        let mut receievr = self.frame_receiver.lock().await;
        if let Some(state) = receievr.recv().await {
            Ok(Response::new(Empty {}))
        } else {
            Err(Status::not_found("no new game state available"))
        }
    }
    async fn input(&self, r: Request<Empty>) -> Result<Response<Empty>, Status> {
        self.next_frame_sender
            .send(NextFrame {})
            .await
            .map_err(|e| Status::unknown(format!("{e:?}")))?;
        Ok(Response::new(Empty {}))
    }
}
