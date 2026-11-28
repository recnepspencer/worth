//! Replaying terminal notification custody does not reopen external dispatch.
use std::net::SocketAddr;

use bank_http_adapter::BankHttpMutationOutcome;
use bank_user_node::BankUserNodeMutationOutcome;

pub async fn assert_original_notice_replays(
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
            response: BankHttpMutationOutcome::RecordedRailRecoveryReady { .. }
        }
    ));
}
