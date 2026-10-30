//! Closed historical correction across independently authenticated node and Bank processes.
pub use super::*;
pub use bank_http_adapter::BankHttpCorrectionSource;
pub use bank_http_adapter::{BankHttpAccountActivityItem, BankHttpAccountActivityPageOutcome};
#[path = "recorded_correction/reapplication.rs"]
pub mod reapplication;

pub fn assert_original_preserved<'a>(
    before: &[BankHttpAccountActivityItem],
    after: &'a [BankHttpAccountActivityItem],
    original_journal: &str,
    reversal: i64,
) -> &'a str {
    assert_eq!(after.len(), before.len() + 1);
    let reversals = after
        .iter()
        .filter(|entry| entry.purpose == BankHttpPostingPurpose::Reversal)
        .collect::<Vec<_>>();
    assert_eq!(reversals.len(), 1);
    assert_eq!(reversals[0].amount_minor, reversal);
    assert_eq!(reversals[0].reversal_of.as_deref(), Some(original_journal));
    assert_eq!(
        after
            .iter()
            .filter(|entry| entry.purpose != BankHttpPostingPurpose::Reversal)
            .cloned()
            .collect::<Vec<_>>(),
        before
    );
    &reversals[0].journal
}

pub fn controls() -> serde_json::Value {
    serde_json::json!({"deadline_milliseconds": 5_000, "maximum_history_work": 128})
}

pub fn create(custody: &str, key: &str, operation: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "command": "create", "request_id": custody, "custody_id": custody,
        "sequence": 1, "controls": controls(), "idempotency_key": key, "operation": operation })
}

pub fn forwarded(outcome: BankUserNodeMutationOutcome) -> BankHttpMutationOutcome {
    let BankUserNodeMutationOutcome::Forwarded { response } = outcome else {
        panic!("authenticated node must forward the actual Bank disposition: {outcome:?}");
    };
    response
}

pub async fn send(
    world: &TransportProcessWorld,
    key: &str,
    amount: i64,
) -> BankHttpCorrectionSource {
    let request = create(
        key,
        key,
        serde_json::json!({ "operation": "send_money",
        "from": "fixture:101", "recipient": "fixture:1", "amount_minor_units": amount }),
    );
    let outcome = forwarded(node_mutation(&world.client, world.peer_address, &request).await);
    let BankHttpMutationOutcome::Completed { source, .. } = outcome else {
        panic!("ordinary source transfer must complete: {outcome:?}");
    };
    close(world, key, 2).await;
    source
}

pub async fn close(world: &TransportProcessWorld, custody: &str, sequence: u64) {
    let request = serde_json::json!({ "command": "close", "request_id": format!("close-{custody}"),
        "custody_id": custody, "sequence": sequence, "controls": { "deadline_milliseconds": 5000 } });
    assert!(matches!(
        forwarded(node_mutation(&world.client, world.peer_address, &request).await),
        BankHttpMutationOutcome::Closed { .. }
    ));
}

pub async fn balance(
    world: &TransportProcessWorld,
    address: SocketAddr,
    account: &str,
    request_id: &str,
) -> i64 {
    match account_summary_with_request_id(&world.client, address, account, request_id).await {
        BankUserNodeAccountSummaryOutcome::Forwarded {
            response: BankHttpAccountSummaryOutcome::Delivered { summary, .. },
        } => summary.current_balance_minor,
        other => panic!("current owner balance: {other:?}"),
    }
}

pub async fn activity(
    world: &TransportProcessWorld,
    address: SocketAddr,
    account: &str,
    request_id: &str,
) -> Vec<BankHttpAccountActivityItem> {
    let outcome: BankUserNodeAccountActivityPageOutcome = post_node(
        &world.client,
        address,
        "/v1/queries/account-activity/page",
        &serde_json::json!({
            "request_id": request_id, "account": account, "controls": request_controls(16),
        }),
    )
    .await;
    match outcome {
        BankUserNodeAccountActivityPageOutcome::Forwarded {
            response:
                BankHttpAccountActivityPageOutcome::Delivered {
                    activity,
                    continuation: None,
                    ..
                },
        } => activity.entries,
        other => panic!("complete current activity: {other:?}"),
    }
}
