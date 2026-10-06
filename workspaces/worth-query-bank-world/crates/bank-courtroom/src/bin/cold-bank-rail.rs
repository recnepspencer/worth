use std::io::{BufRead, Write};

use bank_external_rail::{
    RailCompletionDeliveryConfiguration, RailProtocolSupportProfile, RailServer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut server = RailServer::bind_with_protocol_support(
        "127.0.0.1:0".parse()?,
        RailProtocolSupportProfile::Current,
    )
    .await?;
    let address = server.local_addr()?;
    let control = server.test_control_addr()?;
    println!("LISTENING {address} TEST_CONTROL {control}");
    std::io::stdout().flush()?;
    let mode = std::env::args().nth(3);
    if mode.as_deref() == Some("--completion-config-stdin") {
        let mut line = String::new();
        if std::io::stdin().lock().read_line(&mut line)? == 0 {
            return Err("completion installation was not supplied".into());
        }
        let configuration: RailCompletionDeliveryConfiguration = serde_json::from_str(&line)?;
        server
            .install_completion_delivery(configuration)
            .map_err(|denial| format!("completion installation denied: {denial:?}"))?;
        println!("COMPLETION_DELIVERY_READY");
        std::io::stdout().flush()?;
    }
    if matches!(
        mode.as_deref(),
        Some("--completion-config-stdin" | "--control-stdin")
    ) {
        let close = tokio::task::spawn_blocking(|| {
            let mut line = String::new();
            loop {
                line.clear();
                match std::io::stdin().lock().read_line(&mut line) {
                    Ok(0) => return,
                    Ok(_) if line.trim_end() == "CLOSE" => return,
                    Ok(_) => {}
                    Err(_) => return,
                }
            }
        });
        let posture = server
            .serve_until(async move {
                let _ = close.await;
            })
            .await?;
        println!("CLOSING {}", serde_json::to_string(&posture)?);
        std::io::stdout().flush()?;
    } else {
        server.serve().await?;
    }
    Ok(())
}
