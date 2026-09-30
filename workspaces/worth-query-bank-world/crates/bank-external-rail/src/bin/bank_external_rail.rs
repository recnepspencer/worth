//! Binary entry point for the Bank external rail: a real TCP process,
//! separate from any Query runtime, with controllable exit-proof faults.

use std::io::{BufRead, Write};
use std::net::SocketAddr;

use bank_external_rail::{
    RailCompletionDeliveryConfiguration, RailProtocolSupportProfile, RailServer,
};

#[tokio::main]
async fn main() {
    let bind_addr = parse_bind_addr();
    let protocol_support = parse_protocol_support();
    let mut server = RailServer::bind_with_protocol_support(bind_addr, protocol_support)
        .await
        .unwrap_or_else(|error| {
            eprintln!("bank-external-rail: failed to bind {bind_addr}: {error}");
            std::process::exit(2);
        });
    let local_addr = server
        .local_addr()
        .expect("bank-external-rail: bound listener reports its own address");
    let test_control_addr = server
        .test_control_addr()
        .expect("bank-external-rail: bound test-control listener reports its own address");

    println!("LISTENING {local_addr} TEST_CONTROL {test_control_addr}");
    std::io::stdout()
        .flush()
        .expect("bank-external-rail: stdout is writable at startup");

    let control = std::env::args().nth(3);
    if control.as_deref() == Some("--completion-config-stdin") {
        let mut line = String::new();
        if std::io::stdin()
            .lock()
            .read_line(&mut line)
            .ok()
            .filter(|read| *read > 0)
            .is_none()
        {
            eprintln!("bank-external-rail: completion delivery installation unavailable");
            std::process::exit(2);
        }
        let configuration: RailCompletionDeliveryConfiguration = serde_json::from_str(&line)
            .unwrap_or_else(|_| {
                eprintln!("bank-external-rail: invalid completion delivery installation");
                std::process::exit(2);
            });
        server
            .install_completion_delivery(configuration)
            .unwrap_or_else(|denial| {
                eprintln!(
                    "bank-external-rail: completion delivery installation denied: {denial:?}"
                );
                std::process::exit(2);
            });
        println!("COMPLETION_DELIVERY_READY");
        std::io::stdout()
            .flush()
            .expect("rail installation posture is writable");
    }

    if matches!(
        control.as_deref(),
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
            .await
            .unwrap_or_else(|error| {
                eprintln!("bank-external-rail: listener failed: {error}");
                std::process::exit(1);
            });
        println!(
            "CLOSING {}",
            serde_json::to_string(&posture).expect("bounded delivery posture serializes")
        );
        std::io::stdout()
            .flush()
            .expect("close posture is writable");
    } else {
        let error = server.serve().await.unwrap_err();
        eprintln!("bank-external-rail: listener failed: {error}");
        std::process::exit(1);
    }
}

fn parse_protocol_support() -> RailProtocolSupportProfile {
    let value = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "current".to_owned());
    RailProtocolSupportProfile::parse_command_line(&value).unwrap_or_else(|| {
        eprintln!("bank-external-rail: invalid protocol support profile: {value}");
        std::process::exit(2);
    })
}

fn parse_bind_addr() -> SocketAddr {
    std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:0".to_string())
        .parse()
        .unwrap_or_else(|error| {
            eprintln!("bank-external-rail: invalid bind address argument: {error}");
            std::process::exit(2);
        })
}
