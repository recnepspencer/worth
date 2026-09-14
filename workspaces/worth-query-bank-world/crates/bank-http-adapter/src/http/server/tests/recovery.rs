use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use bank_domain::estate::{DeathNoticeId, DeathNoticeStatus, EstateCaseId};
use bank_domain::model::{AccountId, BankPrincipalId};

use super::super::super::protocol::{
    BankHttpDenialKind, BankHttpEstateNotificationOutcome, BankHttpRecoveryInspectionOutcome,
    BankHttpRecoveryPosture,
};
use super::fixture::application;
use super::{bind_application, credential_json, BankHttpServerConfiguration};

#[tokio::test]
async fn authority_remains_behind_one_opaque_transport_handle() {
    let server = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let action = recovery_action();
    let notified = post_typed::<BankHttpEstateNotificationOutcome>(
        &client,
        server.local_address(),
        "/v1/estate/notify-death",
        &notification_request(&action),
    )
    .await;
    let recovery = match notified {
        BankHttpEstateNotificationOutcome::Applied { recovery, .. } => recovery,
        other => panic!("notification did not commit: {other:?}"),
    };
    assert_recovery_inspects(&client, server.local_address(), &action, &recovery).await;
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn lost_notification_response_replays_the_exact_opaque_handle() {
    let server = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let action = recovery_action();
    let first = notify(&client, server.local_address(), &action, "notification-a").await;
    let replay = notify(&client, server.local_address(), &action, "notification-b").await;
    let (first_commit, first_recovery) = applied_notification(first);
    let (replay_commit, replay_recovery) = applied_notification(replay);
    assert_eq!(replay_commit, first_commit);
    assert_eq!(replay_recovery, first_recovery);
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn expired_recovery_handle_opens_no_inspection_door() {
    let server = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral().with_opaque_handle_lifetime(Duration::ZERO),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let action = recovery_action();
    let notified = notify(
        &client,
        server.local_address(),
        &action,
        "expiring-notification",
    )
    .await;
    let (_, recovery) = applied_notification(notified);
    let inspected = inspect_recovery(&client, server.local_address(), &action, &recovery).await;
    assert!(matches!(
        inspected,
        BankHttpRecoveryInspectionOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::Stale
    ));
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn saturated_recovery_registry_rejects_before_the_domain_effect() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let server = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral()
            .with_opaque_handle_capacity(NonZeroUsize::new(1).unwrap()),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let first = recovery_action();
    applied_notification(notify(&client, server.local_address(), &first, "fill-registry").await);

    let alternate = alternate_recovery_action();
    let denied = notify_with_key(
        &client,
        server.local_address(),
        &alternate,
        "saturated-notification",
        "alternate-notification-key",
    )
    .await;
    assert!(
        matches!(
            &denied,
            BankHttpEstateNotificationOutcome::Denied { denial, .. }
                if denial.kind == BankHttpDenialKind::Saturated
        ),
        "unexpected saturation outcome: {denied:?}"
    );
    assert_eq!(
        application
            .death_notice_status(EstateCaseId::new(4).unwrap())
            .await,
        DeathNoticeStatus::Reported
    );
    server.shutdown().await.expect("server should shut down");
}

async fn assert_recovery_inspects(
    client: &reqwest::Client,
    address: std::net::SocketAddr,
    action: &RecoveryAction,
    recovery: &str,
) {
    let inspected = inspect_recovery(client, address, action, recovery).await;
    assert!(matches!(
        inspected,
        BankHttpRecoveryInspectionOutcome::Inspected {
            posture: BankHttpRecoveryPosture::Reconcilable,
            ..
        }
    ));
}

struct RecoveryAction {
    estate: String,
    notice: String,
    subject: String,
}

fn recovery_action() -> RecoveryAction {
    RecoveryAction {
        estate: EstateCaseId::new(3).unwrap().canonical_text(),
        notice: DeathNoticeId::new(12).unwrap().canonical_text(),
        subject: BankPrincipalId::new(2).unwrap().canonical_text(),
    }
}

fn alternate_recovery_action() -> RecoveryAction {
    RecoveryAction {
        estate: EstateCaseId::new(4).unwrap().canonical_text(),
        notice: DeathNoticeId::new(13).unwrap().canonical_text(),
        subject: BankPrincipalId::new(3).unwrap().canonical_text(),
    }
}

fn notification_request(action: &RecoveryAction) -> serde_json::Value {
    notification_request_with_id(action, "notify-death-http")
}

fn notification_request_with_id(action: &RecoveryAction, request_id: &str) -> serde_json::Value {
    notification_request_with_key(action, request_id, "notify-death-http-key")
}

fn notification_request_with_key(
    action: &RecoveryAction,
    request_id: &str,
    idempotency_key: &str,
) -> serde_json::Value {
    serde_json::json!({
        "protocol": "v1",
        "request_id": request_id,
        "credential": credential_json(),
        "controls": { "deadline_milliseconds": 5_000 },
        "idempotency_key": idempotency_key,
        "estate": action.estate,
        "notice": action.notice,
        "subject": action.subject
    })
}

async fn notify_with_key(
    client: &reqwest::Client,
    address: std::net::SocketAddr,
    action: &RecoveryAction,
    request_id: &str,
    idempotency_key: &str,
) -> BankHttpEstateNotificationOutcome {
    post_typed(
        client,
        address,
        "/v1/estate/notify-death",
        &notification_request_with_key(action, request_id, idempotency_key),
    )
    .await
}

async fn notify(
    client: &reqwest::Client,
    address: std::net::SocketAddr,
    action: &RecoveryAction,
    request_id: &str,
) -> BankHttpEstateNotificationOutcome {
    post_typed(
        client,
        address,
        "/v1/estate/notify-death",
        &notification_request_with_id(action, request_id),
    )
    .await
}

fn applied_notification(
    outcome: BankHttpEstateNotificationOutcome,
) -> (
    super::super::super::protocol::BankHttpCommitDescription,
    String,
) {
    match outcome {
        BankHttpEstateNotificationOutcome::Applied {
            commit, recovery, ..
        } => (commit, recovery),
        other => panic!("notification did not commit: {other:?}"),
    }
}

fn recovery_request(
    _action: &RecoveryAction,
    request_id: &str,
    recovery: &str,
) -> serde_json::Value {
    serde_json::json!({
        "protocol": "v1",
        "request_id": request_id,
        "credential": credential_json(),
        "controls": { "deadline_milliseconds": 5_000 },
        "recovery": recovery
    })
}

async fn inspect_recovery(
    client: &reqwest::Client,
    address: std::net::SocketAddr,
    action: &RecoveryAction,
    recovery: &str,
) -> BankHttpRecoveryInspectionOutcome {
    post_typed(
        client,
        address,
        "/v1/recovery/inspect",
        &recovery_request(action, "inspect-recovery-http", recovery),
    )
    .await
}

async fn post_typed<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    address: std::net::SocketAddr,
    path: &str,
    request: &serde_json::Value,
) -> T {
    client
        .post(format!("http://{address}{path}"))
        .json(request)
        .send()
        .await
        .expect("typed HTTP request should complete")
        .json()
        .await
        .expect("typed HTTP response should decode")
}
