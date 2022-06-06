pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use pb::bot_api_service_client::BotApiServiceClient;
use tokio_stream::wrappers::UnboundedReceiverStream;

use tokio::sync::mpsc;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

#[tokio::main]
async fn main() {
    let channel = Channel::from_static("http://localhost:50051")
        .connect()
        .await
        .unwrap();

    let mut client = BotApiServiceClient::with_interceptor(channel, |mut req: Request<()>| {
        let player_id: MetadataValue<_> = "test_player".parse().unwrap();
        req.metadata_mut().insert("player_id", player_id);
        Ok(req)
    });

    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = client
        .connect_bot(UnboundedReceiverStream::new(rx))
        .await
        .unwrap()
        .into_inner();
    while let Some(message) = stream.message().await.unwrap() {
        let state = String::from_utf8(message.content).unwrap();
        println!("{}", state);
        let response = r#"{ moves: [] }"#;
        tx.send(pb::PlayerRequestResponse {
            request_id: message.request_id,
            content: response.as_bytes().to_vec(),
        })
        .unwrap();
    }
    std::mem::drop(tx);
    // for clean exit
    std::mem::drop(client);
}
