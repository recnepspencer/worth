use std::io::Write;

use bank_external_rail::{RailProtocolSupportProfile, RailServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = RailServer::bind_with_protocol_support(
        "127.0.0.1:0".parse()?,
        RailProtocolSupportProfile::Current,
    )
    .await?;
    let address = server.local_addr()?;
    let control = server.test_control_addr()?;
    println!("LISTENING {address} TEST_CONTROL {control}");
    std::io::stdout().flush()?;
    server.serve().await?;
    Ok(())
}
