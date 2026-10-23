//! An authenticated external notice cannot borrow another operation's journal inverse.
use std::net::SocketAddr;

use bank_external_rail::{inquire_completed_effect_count, inquire_dispatch_contact_count};
use bank_http_adapter::BankHttpCorrectionSource;
use bank_http_adapter::{
    BankHttpAccountActivityItem, BankHttpAccountActivityPageOutcome, BankHttpDenialKind,
    BankHttpMutationOutcome,
};
use bank_user_node::{BankUserNodeAccountActivityPageOutcome, BankUserNodeMutationOutcome};

pub(super) async fn assert_original_notice_replays(
    client: &reqwest::Client,
    node: SocketAddr,
    original: &serde_json::Value,
) {
    let mut request = original.clone();
    request["request_id"] = serde_json::json!("callback-key-replay");
    let replay = super::super::post_node::<bank_user_node::BankUserNodeEstateNotificationOutcome>(
        client,
        node,
        "/v1/estate/notify-death",
        &request,
    )
    .await;
    let closed = super::super::recovery::close_notification_source(client, node, replay).await;
    assert!(matches!(
        closed,
        BankUserNodeMutationOutcome::Forwarded {
            response: BankHttpMutationOutcome::RailRecoveryReady { .. }
        }
    ));
}

pub(super) async fn assert_completed_notice_refuses_journal_inverse(
    client: &reqwest::Client,
    node: SocketAddr,
    rail: SocketAddr,
    source: &BankHttpCorrectionSource,
    original_world: (u64, u64),
) {
    assert_eq!(
        (source.world_owner, source.world_commit),
        original_world,
        "correction selects the actual authenticated external occurrence"
    );
    let contacts = inquire_dispatch_contact_count(rail, super::TIMEOUT)
        .await
        .unwrap();
    let performed = inquire_completed_effect_count(rail, super::TIMEOUT)
        .await
        .unwrap();
    let before = activity(client, node, "notice-correction-before").await;
    let request = serde_json::json!({
        "command": "create", "request_id": "completed-notice-correction",
        "custody_id": "completed-notice-correction", "sequence": 1,
        "controls": {"deadline_milliseconds": 5000, "maximum_history_work": 128},
        "idempotency_key": "completed-notice-correction",
        "operation": {"operation": "correct_send_money", "source": source},
    });
    let outcome = super::super::node_mutation(client, node, &request).await;
    assert!(
        matches!(outcome,
            BankUserNodeMutationOutcome::Forwarded {
                response: BankHttpMutationOutcome::Rejected { ref denial, .. }
            } if denial.kind == BankHttpDenialKind::PermissionDenied
        ),
        "completed external notice has no installed journal inverse: {outcome:?}"
    );
    assert_eq!(
        activity(client, node, "notice-correction-after").await,
        before
    );
    assert_eq!(
        inquire_dispatch_contact_count(rail, super::TIMEOUT)
            .await
            .unwrap(),
        contacts
    );
    assert_eq!(
        inquire_completed_effect_count(rail, super::TIMEOUT)
            .await
            .unwrap(),
        performed
    );
}

async fn activity(
    client: &reqwest::Client,
    node: SocketAddr,
    request_id: &str,
) -> Vec<BankHttpAccountActivityItem> {
    let outcome: BankUserNodeAccountActivityPageOutcome = super::super::post_node(
        client,
        node,
        "/v1/queries/account-activity/page",
        &serde_json::json!({
            "request_id": request_id, "account": "fixture:101",
            "controls": super::super::request_controls(16),
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
        other => panic!("authenticated notice issuer's current account journal: {other:?}"),
    }
}
