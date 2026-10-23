//! Historical estate correction uses fresh authority across the real process boundary.
use super::*;
use bank_http_adapter::{BankHttpAccountStatus, BankHttpAccountSummary};
use worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceCoordinates;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the real Docker identity provider and cold Bank/user-node processes"]
async fn closed_estate_freeze_corrects_until_its_actual_capability_is_revoked() {
    let world = TransportProcessWorld::start().await;
    world.authenticate_participants().await;
    let original = account(&world, world.primary_address, "fixture:100", "original").await;
    let foreign = account(&world, world.peer_address, "fixture:101", "foreign").await;
    assert_eq!(original.status, BankHttpAccountStatus::Open);

    let first = freeze_and_close(&world, "first-freeze").await;
    assert_status(
        &world,
        BankHttpAccountStatus::Frozen,
        &original,
        "first-frozen",
    )
    .await;
    let corrected = complete_and_close(
        &world,
        "restore-first",
        serde_json::json!({"operation":"correct_estate_freeze", "source":first}),
    )
    .await;
    assert_ne!(corrected, first, "correction publishes its own occurrence");
    assert_status(&world, BankHttpAccountStatus::Open, &original, "restored").await;

    let second = freeze_and_close(&world, "second-freeze").await;
    assert_ne!(
        second, first,
        "the later freeze is a distinct recorded source"
    );
    assert_status(
        &world,
        BankHttpAccountStatus::Frozen,
        &original,
        "second-frozen",
    )
    .await;
    complete_and_close(
        &world,
        "revoke-freeze-grant",
        serde_json::json!({"operation":"revoke_estate_capability", "estate":3, "grant":26}),
    )
    .await;
    let denied = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &create(
                "restore-revoked",
                serde_json::json!({
                    "operation":"correct_estate_freeze", "source":second,
                }),
            ),
        )
        .await,
    );
    assert!(
        matches!(denied,
        BankHttpMutationOutcome::Rejected { ref denial, .. }
        if denial.kind == BankHttpDenialKind::PermissionDenied),
        "fresh capability admission must refuse the historical correction: {denied:?}"
    );
    assert_status(
        &world,
        BankHttpAccountStatus::Frozen,
        &original,
        "revoked-frozen",
    )
    .await;
    assert_eq!(
        account(&world, world.peer_address, "fixture:101", "foreign-after").await,
        foreign,
        "estate correction and revocation preserve the unrelated account",
    );
    world.shutdown().await;
}

async fn freeze_and_close(
    world: &TransportProcessWorld,
    key: &str,
) -> WorthQueryCorrectionSourceCoordinates {
    complete_and_close(
        world,
        key,
        serde_json::json!({
            "operation":"freeze_estate_account", "estate":3, "account":"fixture:100",
        }),
    )
    .await
}

async fn complete_and_close(
    world: &TransportProcessWorld,
    key: &str,
    operation: serde_json::Value,
) -> WorthQueryCorrectionSourceCoordinates {
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

fn create(key: &str, operation: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "command":"create", "request_id":key, "custody_id":key, "sequence":1,
        "controls":{"deadline_milliseconds":5_000, "maximum_history_work":128},
        "idempotency_key":key, "operation":operation,
    })
}

fn forwarded(outcome: BankUserNodeMutationOutcome) -> BankHttpMutationOutcome {
    let BankUserNodeMutationOutcome::Forwarded { response } = outcome else {
        panic!("authenticated node forwards the real Bank outcome: {outcome:?}");
    };
    response
}

async fn assert_status(
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

async fn account(
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
