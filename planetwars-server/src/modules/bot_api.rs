pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use std::net::SocketAddr;

use tonic;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub struct BotApiServer {}

#[tonic::async_trait]
impl pb::test_service_server::TestService for BotApiServer {
    async fn greet(&self, req: Request<pb::Hello>) -> Result<Response<pb::HelloResponse>, Status> {
        Ok(Response::new(pb::HelloResponse {
            response: format!("hallo {}", req.get_ref().hello_message),
        }))
    }
}

pub async fn run_bot_api() {
    let server = BotApiServer {};
    let addr = SocketAddr::from(([127, 0, 0, 1], 50051));
    Server::builder()
        .add_service(pb::test_service_server::TestServiceServer::new(server))
        .serve(addr)
        .await
        .unwrap()
}
