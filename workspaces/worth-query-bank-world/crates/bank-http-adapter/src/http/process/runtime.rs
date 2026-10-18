use std::error::Error;
use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};

use crate::{AuthentikBankIdentity, BankHttpServerBinding, BankHttpServerConfiguration};

use super::BankHttpProcessConfiguration;

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
    let binding =
        BankHttpServerBinding::bind(BankHttpServerConfiguration::local_ephemeral()).await?;
    write_posture("bound", binding.local_address())?;
    let configuration: BankHttpProcessConfiguration = read_json_line()?;
    let identity = install_identity_on_dedicated_stack(configuration).await?;
    let (server, _mutation_continuation) = binding.install(identity)?.into_parts();
    let server = server.activate();
    write_posture("ready", server.local_address())?;
    let mut server = server;
    loop {
        let command = tokio::task::spawn_blocking(read_json_line::<ProcessCommand>).await??;
        if command.command != "shutdown" {
            write_posture("command_refused", server.local_address())?;
            continue;
        }
        let address = server.local_address();
        match server.shutdown().await {
            Ok(_) => break,
            Err(failure) => match failure.into_server() {
                Ok(retained) => {
                    server = retained;
                    write_posture("shutdown_refused", address)?;
                }
                Err(failure) => return Err(Box::new(failure)),
            },
        }
    }
    if _mutation_continuation.finish().await.is_err() {
        return Err("mutation worker finish refused".into());
    }

    Ok(())
}

async fn install_identity_on_dedicated_stack(
    configuration: BankHttpProcessConfiguration,
) -> Result<AuthentikBankIdentity, Box<dyn Error + Send + Sync>> {
    const INSTALLATION_STACK_BYTES: usize = 16 * 1024 * 1024;

    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
    std::thread::Builder::new()
        .name("bank-world-installation".to_owned())
        .stack_size(INSTALLATION_STACK_BYTES)
        .spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| Box::new(error) as Box<dyn Error + Send + Sync>)
                .and_then(|runtime| {
                    runtime
                        .block_on(configuration.install_identity())
                        .map_err(|error| Box::new(error) as Box<dyn Error + Send + Sync>)
                });
            let _ = result_sender.send(result);
        })?;
    result_receiver.await.map_err(|_| {
        Box::<dyn Error + Send + Sync>::from(
            "Bank world installation thread stopped before reporting its result",
        )
    })?
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
