use std::error::Error;
use std::io::{BufRead, Write};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{BankHttpServerBinding, BankHttpServerConfiguration};

use super::{BankHttpProcessCloseRefused, BankHttpProcessConfiguration};

#[derive(Deserialize)]
struct ProcessCommand {
    command: String,
}

#[derive(Serialize)]
struct ProcessPosture<'a> {
    state: &'a str,
    process_id: u32,
    address: String,
}

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let lifecycle = lifecycle_funding()?;
    let binding =
        BankHttpServerBinding::bind(BankHttpServerConfiguration::local_ephemeral()).await?;
    write_posture("bound", binding.local_address())?;
    let configuration: BankHttpProcessConfiguration = read_json_line()?;
    let identity = Arc::new(configuration.install_identity().await?);
    let installation = binding
        .install_shared(Arc::clone(&identity))
        .map_err(|cause| BankHttpProcessCloseRefused::Installation {
            identity: Arc::clone(&identity),
            cause,
        })?;
    let (server, mutation_continuation) = installation.into_parts();
    let server = server.activate();
    let server_address = server.local_address();
    if let Err(cause) = write_posture("ready", server_address) {
        return Err(Box::new(BankHttpProcessCloseRefused::Running {
            identity,
            server,
            mutations: mutation_continuation,
            cause,
        }));
    }
    let mut server = server;
    let close = loop {
        match shutdown_requested(server.local_address()).await {
            Ok(false) => continue,
            Ok(true) => {}
            Err(cause) => {
                return Err(Box::new(BankHttpProcessCloseRefused::Running {
                    identity,
                    server,
                    mutations: mutation_continuation,
                    cause,
                }))
            }
        };
        let address = server.local_address();
        match server.shutdown().await {
            Ok(close) => break close,
            Err(failure) => match failure.into_server() {
                Ok(retained) => {
                    server = retained;
                    if let Err(cause) = write_posture("shutdown_refused", address) {
                        return Err(Box::new(BankHttpProcessCloseRefused::Running {
                            identity,
                            server,
                            mutations: mutation_continuation,
                            cause,
                        }));
                    }
                }
                Err(failure) => {
                    return Err(Box::new(BankHttpProcessCloseRefused::Workers {
                        identity,
                        mutations: mutation_continuation,
                        failure,
                    }))
                }
            },
        }
    };
    super::closure::finish(
        identity,
        mutation_continuation,
        close,
        &worth_execution::ExecutionRequest::leased(&lifecycle),
    )
    .await?;
    Ok(())
}

#[expect(
    clippy::disallowed_methods,
    reason = "async-runtime async-blocking=1: Bank process reads command input through its Tokio I/O executor."
)]
async fn shutdown_requested(
    address: std::net::SocketAddr,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let command = tokio::task::spawn_blocking(read_json_line::<ProcessCommand>).await??;
    if command.command == "shutdown" {
        return Ok(true);
    }
    write_posture("command_refused", address)?;
    Ok(false)
}

fn lifecycle_funding(
) -> Result<worth_execution::ExecutionResourceLease<'static>, Box<dyn Error + Send + Sync>> {
    use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 << 20), None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .map_err(|denial| {
        std::io::Error::other(format!("Bank Home lifecycle funding: {denial:?}")).into()
    })
}

fn read_json_line<T>() -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: serde::de::DeserializeOwned,
{
    let mut line = String::new();
    let read = std::io::stdin().lock().read_line(&mut line)?;
    if read == 0 {
        return Err("Bank HTTP server process input closed".into());
    }
    serde_json::from_str(&line).map_err(Into::into)
}

fn write_posture(
    state: &str,
    address: std::net::SocketAddr,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let posture = ProcessPosture {
        state,
        process_id: std::process::id(),
        address: address.to_string(),
    };
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &posture)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}
