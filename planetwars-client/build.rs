extern crate tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&["../proto/bot_api.proto"], &["../proto"])?;
    Ok(())
}
