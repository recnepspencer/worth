use std::net::SocketAddr;

use bank_external_rail::test_control::FaultScript;
use bank_http_adapter::{
    BankHttpCommitDisposition, BankHttpDenialKind, BankHttpRecoveryRetryDisposition,
};
use bank_user_node::{
    BankUserNodeEstateNotificationOutcome, BankUserNodeRecoveryInspectionOutcome,
    BankUserNodeRecoverySafeRetryOutcome,
};

use super::{post_node, world::TransportProcessWorld};

pub async fn assert_opaque_recovery_is_owned_by_the_authenticated_specialist(
    world: &TransportProcessWorld,
) {
    let client = &world.client;
    let primary = world.primary_address;
    let specialist = world.peer_address;
    let admission_before = world.rail_admission_count().await;
    let contact_before = world.rail_contact_count().await;
    let completed_before = world.rail_completed_effect_count().await;
    world
        .select_rail_fault(FaultScript::CommitThenLoseResponse)
        .await;
    let request = serde_json::json!({
        "request_id": "process-notify-death",
        "controls": { "deadline_milliseconds": 5_000 },
        "idempotency_key": "process-notify-death-key",
        "estate": "fixture:3",
        "notice": "fixture:12",
        "subject": "fixture:1"
    });
    let first = post_node::<BankUserNodeEstateNotificationOutcome>(
        client,
        specialist,
        "/v1/estate/notify-death",
        &request,
    )
    .await;
    assert!(matches!(
        first,
        BankUserNodeEstateNotificationOutcome::Forwarded {
            response: bank_http_adapter::BankHttpEstateNotificationOutcome::Applied {
                disposition: BankHttpCommitDisposition::Committed,
                ..
            }
        }
    ));
    assert_eq!(world.rail_admission_count().await, admission_before + 1);
    assert_eq!(world.rail_contact_count().await, contact_before + 1);
    assert_eq!(
        world.rail_completed_effect_count().await,
        completed_before + 1
    );
    let notified = post_node::<BankUserNodeEstateNotificationOutcome>(
        client,
        specialist,
        "/v1/estate/notify-death",
        &request,
    )
    .await;
    let recovery = match notified {
        BankUserNodeEstateNotificationOutcome::Forwarded {
            response:
                bank_http_adapter::BankHttpEstateNotificationOutcome::Applied {
                    disposition: BankHttpCommitDisposition::AlreadyCommitted,
                    recovery,
                    ..
                },
        } => recovery.expect("live commit replay must retain its recovery token"),
        other => panic!("specialist notification did not commit: {other:?}"),
    };
    assert_eq!(
        world.rail_admission_count().await,
        admission_before + 1,
        "exact HTTP replay must not add a rail admission"
    );
    assert_eq!(
        world.rail_contact_count().await,
        contact_before + 1,
        "exact HTTP replay must not send another dispatch frame"
    );
    let crossed = inspect(client, primary, "cross-user-recovery", &recovery).await;
    assert!(matches!(
        crossed,
        BankUserNodeRecoveryInspectionOutcome::Forwarded {
            response: bank_http_adapter::BankHttpRecoveryInspectionOutcome::Denied {
                denial,
                ..
            }
        } if denial.kind == BankHttpDenialKind::Stale
    ));
    let inspected = inspect(client, specialist, "inspect-recovery", &recovery).await;
    assert!(matches!(
        inspected,
        BankUserNodeRecoveryInspectionOutcome::Forwarded {
            response: bank_http_adapter::BankHttpRecoveryInspectionOutcome::Inspected {
                posture: bank_http_adapter::BankHttpRecoveryPosture::Reconcilable,
                ..
            }
        }
    ));
    let foreign_retry = retry(client, primary, "cross-user-retry", &recovery).await;
    assert!(matches!(
        foreign_retry,
        BankUserNodeRecoverySafeRetryOutcome::Forwarded {
            response: bank_http_adapter::BankHttpRecoverySafeRetryOutcome::Denied {
                denial,
                ..
            }
        } if denial.kind == BankHttpDenialKind::Stale
    ));
    assert_eq!(world.rail_contact_count().await, contact_before + 1);
    let retried = retry(client, specialist, "specialist-retry", &recovery).await;
    assert!(matches!(
        retried,
        BankUserNodeRecoverySafeRetryOutcome::Forwarded {
            response: bank_http_adapter::BankHttpRecoverySafeRetryOutcome::Applied {
                disposition: BankHttpRecoveryRetryDisposition::Retried,
                fresh_attempt: true,
                external_completion: true,
                ..
            }
        }
    ));
    assert_eq!(world.rail_contact_count().await, contact_before + 2);
    assert_eq!(world.rail_admission_count().await, admission_before + 1);
    assert_eq!(
        world.rail_completed_effect_count().await,
        completed_before + 1
    );
    let replay = retry(client, specialist, "specialist-replay", &recovery).await;
    assert!(matches!(
        replay,
        BankUserNodeRecoverySafeRetryOutcome::Forwarded {
            response: bank_http_adapter::BankHttpRecoverySafeRetryOutcome::Applied {
                disposition: BankHttpRecoveryRetryDisposition::AlreadyRetried,
                fresh_attempt: true,
                external_completion: true,
                ..
            }
        }
    ));
    assert_eq!(world.rail_contact_count().await, contact_before + 2);
}

async fn retry(
    client: &reqwest::Client,
    address: SocketAddr,
    request_id: &str,
    recovery: &str,
) -> BankUserNodeRecoverySafeRetryOutcome {
    post_node(
        client,
        address,
        "/v1/recovery/safe-retry",
        &recovery_request(request_id, recovery),
    )
    .await
}

async fn inspect(
    client: &reqwest::Client,
    address: SocketAddr,
    request_id: &str,
    recovery: &str,
) -> BankUserNodeRecoveryInspectionOutcome {
    post_node(
        client,
        address,
        "/v1/recovery/inspect",
        &recovery_request(request_id, recovery),
    )
    .await
}

fn recovery_request(request_id: &str, recovery: &str) -> serde_json::Value {
    serde_json::json!({
        "request_id": request_id,
        "controls": { "deadline_milliseconds": 5_000 },
        "recovery": recovery
    })
}
