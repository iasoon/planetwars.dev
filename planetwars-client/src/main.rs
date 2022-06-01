pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use pb::test_service_client::TestServiceClient;
use pb::{Hello, HelloResponse};
use tonic::Response;

#[tokio::main]
async fn main() {
    let mut client = TestServiceClient::connect("http://localhost:50051")
        .await
        .unwrap();
    let response: Response<HelloResponse> = client
        .greet(Hello {
            hello_message: "robbe".to_string(),
        })
        .await
        .unwrap();
    println!("{}", response.get_ref().response);
}
