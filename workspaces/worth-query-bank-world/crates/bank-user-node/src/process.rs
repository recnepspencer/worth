use std::error::Error;
use std::io::{BufRead, Write};
use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

use crate::{BankUserNode, BankUserNodeBinding, BankUserNodeConfiguration};

#[derive(Deserialize)]
struct ProcessConfiguration {
    issuer: String,
    client_id: String,
    client_secret: String,
    introspection_url: String,
    revocation_url: String,
    bank_server_origin: String,
    maximum_request_concurrency: Option<NonZeroUsize>,
    maximum_live_streams: Option<NonZeroUsize>,
    #[serde(default)]
    cold_certification: bool,
    external_redirect_url: Option<String>,
}

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

struct NodeInstallation {
    configuration: BankUserNodeConfiguration,
    cold_certification: bool,
    external_redirect_url: Option<String>,
}

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let binding = BankUserNodeBinding::bind_local().await?;
    write_posture("bound", binding.local_address())?;
    let installation = read_configuration()?;
    let node = install_node(binding, installation).await?;
    write_posture("ready", node.local_address())?;
    let command = tokio::task::spawn_blocking(read_command).await??;
    if command.command != "shutdown" {
        return Err("unsupported user-node process command".into());
    }
    node.shutdown().await?;
    Ok(())
}

fn read_configuration() -> Result<NodeInstallation, Box<dyn Error + Send + Sync>> {
    let input: ProcessConfiguration = read_json_line()?;
    let mut builder = BankUserNodeConfiguration::builder()
        .issuer(input.issuer)
        .client_id(input.client_id)
        .client_secret(input.client_secret)
        .introspection_url(input.introspection_url)
        .revocation_url(input.revocation_url)
        .bank_server_origin(input.bank_server_origin);
    if let Some(maximum) = input.maximum_request_concurrency {
        builder = builder.maximum_request_concurrency(maximum);
    }
    if let Some(maximum) = input.maximum_live_streams {
        builder = builder.maximum_live_streams(maximum);
    }
    let configuration = builder.build()?;
    Ok(NodeInstallation {
        configuration,
        cold_certification: input.cold_certification,
        external_redirect_url: input.external_redirect_url,
    })
}

fn read_command() -> Result<ProcessCommand, Box<dyn Error + Send + Sync>> {
    read_json_line()
}

fn read_json_line<T>() -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: serde::de::DeserializeOwned,
{
    let mut line = String::new();
    let read = std::io::stdin().lock().read_line(&mut line)?;
    if read == 0 {
        return Err("user-node process input closed".into());
    }
    serde_json::from_str(&line).map_err(Into::into)
}

async fn install_node(
    binding: BankUserNodeBinding,
    installation: NodeInstallation,
) -> Result<BankUserNode, Box<dyn Error + Send + Sync>> {
    if !installation.cold_certification {
        return binding
            .install(installation.configuration)
            .await
            .map_err(Into::into);
    }
    install_cold_node(binding, installation).await
}

async fn install_cold_node(
    binding: BankUserNodeBinding,
    installation: NodeInstallation,
) -> Result<BankUserNode, Box<dyn Error + Send + Sync>> {
    let redirect = installation
        .external_redirect_url
        .ok_or("cold user-node installation requires an external redirect URL")?;
    crate::cold_certification::install(binding, installation.configuration, redirect)
        .await
        .map_err(Into::into)
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
