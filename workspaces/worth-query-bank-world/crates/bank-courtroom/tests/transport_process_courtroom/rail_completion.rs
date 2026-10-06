//! A completed consequence returns from a separate rail process through Bank HTTP into World.

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
use bank_user_node::{BankUserNodeEstateNotificationOutcome, BankUserNodeRecoverySafeRetryOutcome};
use ed25519_dalek::{Signer, SigningKey};
use tokio::sync::oneshot;

use super::identity_world::docker_world::DockerIdentityWorld;
use super::identity_world::fixture::IdentityFixture;
use super::post_node;
use super::process::CourtroomProcess;
use super::world::{
    authenticate_node, external_redirect, node_configuration, server_configuration,
};

#[path = "rail_completion/contact_barrier.rs"]
mod contact_barrier;
#[path = "rail_completion/duplicate.rs"]
mod duplicate;
#[path = "rail_completion/initiating_client.rs"]
mod initiating_client;
#[path = "rail_completion/proxy.rs"]
pub(crate) mod proxy;
use proxy::{correlation_token, CallbackProxy};

const TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const AUDIENCE: &str = "bank-process-court";
pub(super) const RAIL_PUBLIC_KEY: [u8; 32] = [
    234, 74, 108, 99, 226, 156, 82, 10, 190, 245, 80, 123, 19, 46, 197, 249, 149, 71, 118, 174,
    190, 190, 123, 146, 66, 30, 234, 105, 20, 70, 210, 44,
];
pub(super) const BANK_PUBLIC_KEY: [u8; 32] = [
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
    let initiating_client = initiating_client::post_node_holding_response(
        node_address,
        "/v1/estate/notify-death",
        &request,
    )
    .await;
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
    // Callback arrival proves the outbox committed before client disconnect.
    drop(initiating_client);
    let replay_before_callback = post_node::<BankUserNodeEstateNotificationOutcome>(
        &client,
        node_address,
        "/v1/estate/notify-death",
        &request,
    )
    .await;
    let recovery = match replay_before_callback {
        BankUserNodeEstateNotificationOutcome::Forwarded {
            response:
                BankHttpEstateNotificationOutcome::Applied {
                    disposition: BankHttpCommitDisposition::AlreadyCommitted,
                    recovery: Some(recovery),
                    ..
                },
        } => recovery,
        other => panic!("held callback must leave exact recovery ownership: {other:?}"),
    };
    assert!(bank.observe_rail_completion(token).is_none());
    let contacts_before_retry = contact_barrier::count(rail.local_addr()).await;
    assert_eq!(contacts_before_retry, 1, "one initiating rail dispatch");
    let retry_client = client.clone();
    let racing_recovery = recovery.clone();
    let retry = tokio::spawn(async move {
        post_node::<BankUserNodeRecoverySafeRetryOutcome>(
            &retry_client,
            node_address,
            "/v1/recovery/safe-retry",
            &serde_json::json!({
                "request_id": "process-rail-callback-racing-retry",
                "controls": { "deadline_milliseconds": 5_000 },
                "recovery": racing_recovery
            }),
        )
        .await
    });
    contact_barrier::await_next(rail.local_addr(), contacts_before_retry).await;
    proxy.release_first();

    let retry_result = retry
        .await
        .expect("safe retry should return its owner result");
    contact_barrier::assert_racing_retry_result(&retry_result);

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
    assert_eq!(
        contact_barrier::count(rail.local_addr()).await,
        contacts_before_retry + 1,
        "one owner safe retry crossed the rail before callback release"
    );
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
    contact_barrier::assert_post_terminal_retry(
        &client,
        node_address,
        &recovery,
        rail.local_addr(),
        contacts_before_retry + 1,
    )
    .await;

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

    duplicate::assert_exact_callbacks(&client, bank_address, &captured[0]).await;
    let after_duplicates = bank
        .observe_rail_completion(token)
        .expect("duplicates retain the same World terminal");
    assert_eq!(
        after_duplicates.completion_world_commit(),
        terminal.completion_world_commit()
    );
    assert_eq!(
        inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
            .await
            .unwrap(),
        1
    );

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
    let mut altered_meaning = captured[0].clone();
    let signature_start = altered_meaning.len() - 64;
    altered_meaning[signature_start - 1] ^= 1;
    let signature = SigningKey::from_bytes(&[7; 32]).sign(&altered_meaning[..signature_start]);
    altered_meaning[signature_start..].copy_from_slice(&signature.to_bytes());
    let conflict = client
        .post(format!("http://{bank_address}/v1/inbound/rail-completions"))
        .body(altered_meaning.clone())
        .send()
        .await
        .expect("signed conflicting callback should reach actual endpoint");
    assert_eq!(conflict.status(), reqwest::StatusCode::OK);
    let permanent_ack = conflict.bytes().await.unwrap();
    duplicate::assert_permanent_denial(
        &permanent_ack,
        &captured[0],
        &altered_meaning,
        BANK_PUBLIC_KEY,
    );
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
