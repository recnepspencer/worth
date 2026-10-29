//! Historical estate correction uses fresh authority across the real process boundary.
pub use super::*;
pub use bank_http_adapter::BankHttpCorrectionSource;
pub use bank_http_adapter::{BankHttpAccountStatus, BankHttpAccountSummary};

pub async fn freeze_and_close(
    world: &TransportProcessWorld,
    key: &str,
) -> BankHttpCorrectionSource {
    complete_and_close(
        world,
        key,
        serde_json::json!({
            "operation":"freeze_estate_account", "estate":3, "account":"fixture:100",
        }),
    )
    .await
}

pub async fn complete_and_close(
    world: &TransportProcessWorld,
    key: &str,
    operation: serde_json::Value,
) -> BankHttpCorrectionSource {
    let mut outcome =
        forwarded(node_mutation(&world.client, world.peer_address, &create(key, operation)).await);
    while let BankHttpMutationOutcome::Pending { sequence, .. } = outcome {
        outcome = forwarded(
            node_mutation(
                &world.client,
                world.peer_address,
                &serde_json::json!({
                    "command":"resume", "request_id":format!("resume-{key}-{sequence}"),
                    "custody_id":key, "sequence":sequence+1,
                    "controls":{"deadline_milliseconds":5_000, "maximum_history_work":128},
                }),
            )
            .await,
        );
    }
    let BankHttpMutationOutcome::Completed {
        source,
        sequence,
        recovered,
        ..
    } = outcome
    else {
        panic!("ordinary estate command must complete: {outcome:?}");
    };
    assert!(!recovered, "each command is a fresh occurrence");
    let closed = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &serde_json::json!({
                "command":"close", "request_id":format!("close-{key}"), "custody_id":key,
                "sequence":sequence+1, "controls":{"deadline_milliseconds":5_000},
            }),
        )
        .await,
    );
    assert!(
        matches!(closed, BankHttpMutationOutcome::Closed { .. }),
        "the original estate command owner closes normally: {closed:?}"
    );
    source
}

pub fn create(key: &str, operation: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "command":"create", "request_id":key, "custody_id":key, "sequence":1,
        "controls":{"deadline_milliseconds":5_000, "maximum_history_work":128},
        "idempotency_key":key, "operation":operation,
    })
}

pub fn forwarded(outcome: BankUserNodeMutationOutcome) -> BankHttpMutationOutcome {
    let BankUserNodeMutationOutcome::Forwarded { response } = outcome else {
        panic!("authenticated node forwards the real Bank outcome: {outcome:?}");
    };
    response
}

pub async fn assert_status(
    world: &TransportProcessWorld,
    status: BankHttpAccountStatus,
    original: &BankHttpAccountSummary,
    request_id: &str,
) {
    let observed = account(world, world.primary_address, "fixture:100", request_id).await;
    assert_eq!(observed.status, status);
    let mut expected = original.clone();
    expected.status = status;
    assert_eq!(
        observed, expected,
        "only the authored account status changes"
    );
}

pub async fn account(
    world: &TransportProcessWorld,
    owner: SocketAddr,
    account: &str,
    request_id: &str,
) -> BankHttpAccountSummary {
    match account_summary_with_request_id(&world.client, owner, account, request_id).await {
        BankUserNodeAccountSummaryOutcome::Forwarded {
            response: BankHttpAccountSummaryOutcome::Delivered { summary, .. },
        } => summary,
        other => panic!("independent account owner observes the current account: {other:?}"),
    }
}
