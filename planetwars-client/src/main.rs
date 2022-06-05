pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use pb::bot_api_service_client::BotApiServiceClient;
use tokio_stream::wrappers::UnboundedReceiverStream;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let mut client = BotApiServiceClient::connect("http://localhost:50051")
        .await
        .unwrap();

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
