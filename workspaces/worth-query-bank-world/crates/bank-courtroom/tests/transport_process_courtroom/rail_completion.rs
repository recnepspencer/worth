//! A completed consequence returns from a separate rail process through Bank HTTP into World.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use bank_external_rail::test_control::{select_fault, FaultScript};
use bank_external_rail::{
    inquire_completed_effect_count, inquire_completion_delivery_posture,
    RailCompletionDeliveryConfiguration, RailProcessHandle,
};
use bank_http_adapter::{
    BankHttpCommitDisposition, BankHttpEstateNotificationOutcome, BankHttpProcessConfiguration,
    BankHttpServerBinding, BankHttpServerConfiguration, BankRailCompletionServerInstallation,
};
use bank_user_node::BankUserNodeEstateNotificationOutcome;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::oneshot;

use super::identity_world::docker_world::DockerIdentityWorld;
use super::identity_world::fixture::IdentityFixture;
use super::post_node;
use super::process::CourtroomProcess;
use super::world::{
    authenticate_node, external_redirect, node_configuration, server_configuration,
};

#[path = "rail_completion/proxy.rs"]
mod proxy;
use proxy::{correlation_token, CallbackProxy};

const TIMEOUT: Duration = Duration::from_secs(5);
const AUDIENCE: &str = "bank-process-court";
const RAIL_PUBLIC_KEY: [u8; 32] = [
    234, 74, 108, 99, 226, 156, 82, 10, 190, 245, 80, 123, 19, 46, 197, 249, 149, 71, 118, 174,
    190, 190, 123, 146, 66, 30, 234, 105, 20, 70, 210, 44,
];
const BANK_PUBLIC_KEY: [u8; 32] = [
    253, 23, 36, 56, 90, 160, 199, 91, 100, 251, 120, 205, 96, 47, 161, 217, 145, 253, 235, 247,
    107, 19, 197, 142, 215, 2, 234, 200, 53, 233, 246, 24,
];

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn completed_rail_callback_survives_lost_ack_and_publishes_world_once() {
    let mut rail = RailProcessHandle::spawn_awaiting_completion_installation(
        Path::new(env!("CARGO_BIN_EXE_cold-bank-rail")),
        "127.0.0.1:0",
    )
    .expect("separate rail process should bind");
    let (mut node, bound) =
        CourtroomProcess::spawn(Path::new(env!("CARGO_BIN_EXE_cold-bank-user-node")))
            .await
            .expect("user node should bind separately");
    assert_ne!(rail.pid(), std::process::id());
    assert_ne!(rail.pid(), bound.process_id);
    let node_address = node.local_address();
    let redirect = external_redirect(node_address);
    let fixture = IdentityFixture::dynamic(redirect.clone());
    let docker = DockerIdentityWorld::start(&fixture)
        .await
        .expect("cold identity world should start");
    let endpoints = docker
        .wait_until_ready(&fixture)
        .await
        .expect("cold identity world should be ready");
    eprintln!("rail completion court: identity endpoints ready");

    let bank_binding = BankHttpServerBinding::bind(BankHttpServerConfiguration::local_ephemeral())
        .await
        .expect("Bank HTTP callback should bind");
    let bank_address = bank_binding.local_address();
    let configuration: BankHttpProcessConfiguration = serde_json::from_value(server_configuration(
        &fixture,
        &endpoints,
        &redirect,
        rail.local_addr(),
    ))
    .expect("Bank process configuration should parse");
    let identity = Arc::new(
        tokio::time::timeout(Duration::from_secs(330), install_identity(configuration))
            .await
            .expect("Bank identity installation should finish within its 300-second scope"),
    );
    eprintln!("rail completion court: Bank identity installed");
    let bank = bank_binding
        .install_shared_with_rail_completion(
            Arc::clone(&identity),
            BankRailCompletionServerInstallation::new(
                RAIL_PUBLIC_KEY,
                [9; 32],
                AUDIENCE.to_owned(),
                "rail-primary".to_owned(),
                1,
                60,
            )
            .expect("fixed rail verifier and Bank ACK signer should install"),
        )
        .expect("real Bank HTTP callback should start");
    eprintln!("rail completion court: Bank callback bound");
    let mut proxy = CallbackProxy::start(bank_address).await;
    eprintln!("rail completion court: callback proxy bound; installing rail sender");
    rail.install_completion_delivery(
        &RailCompletionDeliveryConfiguration::new(
            format!("http://{}/v1/inbound/rail-completions", proxy.address),
            AUDIENCE.to_owned(),
            "rail-primary".to_owned(),
            1,
            [7; 32],
            BANK_PUBLIC_KEY,
            60,
            2,
            4,
            Duration::from_millis(50),
            Duration::from_secs(5),
            None,
        )
        .expect("bounded rail callback sender should configure"),
    )
    .expect("rail sender must install before dispatch");
    eprintln!("rail completion court: rail sender installed");
    select_fault(
        rail.test_control_addr(),
        FaultScript::CommitThenLoseResponse,
        TIMEOUT,
    )
    .await
    .expect("rail must commit the consequence without a synchronous response");
    eprintln!("rail completion court: response-loss fault installed");
    node.install(node_configuration(
        &fixture,
        &endpoints,
        bank.local_address(),
        &redirect,
    ))
    .await
    .expect("user node should connect to Bank HTTP");
    eprintln!("rail completion court: user node installed");
    let client = reqwest::Client::new();
    authenticate_node(
        &client,
        node_address,
        &endpoints.webdriver_url(),
        &fixture.participants()[1],
    )
    .await;
    eprintln!("rail completion court: user node authenticated");

    let request = serde_json::json!({
        "request_id": "process-rail-callback-notice",
        "controls": { "deadline_milliseconds": 5_000 },
        "idempotency_key": "process-rail-callback-notice-key",
        "estate": "fixture:3",
        "notice": "fixture:12",
        "subject": "fixture:1"
    });
    let initiating_client =
        post_node_holding_response(node_address, "/v1/estate/notify-death", &request).await;
    let first_envelope = tokio::time::timeout(Duration::from_secs(20), proxy.await_first_capture())
        .await
        .expect("rail callback must arrive after the committed dispatch");
    assert_eq!(proxy.captured().len(), 1, "first callback remains held");
    let token = correlation_token(&first_envelope);
    assert!(
        bank.observe_rail_completion(token).is_none(),
        "held callback cannot publish a World terminal"
    );
    assert_eq!(
        inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
            .await
            .expect("rail should retain the in-flight callback")
            .pending,
        1,
        "held callback has no custody ACK"
    );
    // Callback arrival proves the initiating request reached the committed
    // outbox. Lose its response only after that barrier, then release delivery.
    drop(initiating_client);
    proxy.release_first();

    let terminal_wait = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let captured = proxy.captured();
            if let Some(envelope) = captured.first() {
                let token = correlation_token(envelope);
                if let Some(terminal) = bank.observe_rail_completion(token) {
                    let posture = inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
                        .await
                        .expect("rail should report callback custody");
                    if captured.len() >= 2 && posture.pending == 0 {
                        break (envelope.clone(), token, terminal);
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await;
    if terminal_wait.is_err() {
        eprintln!(
            "rail completion court timeout: captured={}, terminal={}, rail={:?}",
            proxy.captured().len(),
            bank.observe_rail_completion(token).is_some(),
            inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT).await,
        );
    }
    let (envelope, token, terminal) =
        terminal_wait.expect("lost ACK must be retried and Query World must become terminal");
    let captured = proxy.captured();
    assert_eq!(captured.len(), 2, "one dropped ACK must cause one retry");
    assert_eq!(captured[0], captured[1], "retry must preserve signed bytes");
    assert_ne!(
        terminal.original_world_commit(),
        terminal.completion_world_commit()
    );
    assert_eq!(
        inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
            .await
            .expect("independent rail should report completed consequence"),
        1
    );

    let replay = post_node::<BankUserNodeEstateNotificationOutcome>(
        &client,
        node_address,
        "/v1/estate/notify-death",
        &request,
    )
    .await;
    assert!(matches!(
        replay,
        BankUserNodeEstateNotificationOutcome::Forwarded {
            response: BankHttpEstateNotificationOutcome::Applied {
                disposition: BankHttpCommitDisposition::AlreadyCommitted,
                ..
            }
        }
    ));
    assert_eq!(proxy.captured().len(), 2, "replay cannot dispatch again");

    let mut corrupted = envelope;
    *corrupted
        .last_mut()
        .expect("signed callback has a signature") ^= 1;
    let denial = client
        .post(format!("http://{bank_address}/v1/inbound/rail-completions"))
        .body(corrupted)
        .send()
        .await
        .expect("malformed callback should reach actual endpoint");
    assert_eq!(denial.status(), reqwest::StatusCode::UNAUTHORIZED);
    let after = bank
        .observe_rail_completion(token)
        .expect("World terminal observation must remain available");
    assert_eq!(
        after.completion_world_commit(),
        terminal.completion_world_commit()
    );
    assert_eq!(after.completion_attempt(), terminal.completion_attempt());
    assert_eq!(
        inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
            .await
            .expect("rail consequence count should remain readable"),
        1
    );

    proxy.shutdown().await;
    node.shutdown().await.expect("user node should stop");
    bank.shutdown().await.expect("Bank HTTP should stop");
    let project = docker.project_name();
    drop(docker);
    DockerIdentityWorld::require_project_absent(&project)
        .expect("courtroom teardown should remove Docker resources");
}

async fn post_node_holding_response(
    address: SocketAddr,
    path: &str,
    body: &serde_json::Value,
) -> TcpStream {
    let body = serde_json::to_vec(body).expect("notice body should encode");
    let mut stream = TcpStream::connect(address)
        .await
        .expect("initiating client should connect to the user node");
    let headers = format!(
        "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .await
        .expect("initiating client should send headers");
    stream
        .write_all(&body)
        .await
        .expect("initiating client should send the complete notice");
    stream
}

async fn install_identity(
    configuration: BankHttpProcessConfiguration,
) -> bank_http_adapter::AuthentikBankIdentity {
    let (sender, receiver) = oneshot::channel();
    std::thread::Builder::new()
        .name("callback-court-installation".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("installation runtime should build");
            let result = runtime.block_on(configuration.install_identity());
            let _ = sender.send(result);
        })
        .expect("Bank installation thread should start");
    receiver
        .await
        .expect("Bank installation should report")
        .expect("Bank world should install")
}
