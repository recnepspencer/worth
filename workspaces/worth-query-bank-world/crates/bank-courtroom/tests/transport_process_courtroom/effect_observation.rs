use std::net::SocketAddr;

use bank_http_adapter::{
    BankHttpAccountActivityItem, BankHttpAccountActivityPageOutcome, BankHttpAccountSummaryOutcome,
};
use bank_user_node::{BankUserNodeAccountActivityPageOutcome, BankUserNodeAccountSummaryOutcome};

use super::{account_summary_with_request_id, request_controls};

pub(super) async fn current_balance(
    client: &reqwest::Client,
    address: SocketAddr,
    request_id: &str,
) -> i64 {
    match account_summary_with_request_id(client, address, "fixture:100", request_id).await {
        BankUserNodeAccountSummaryOutcome::Forwarded {
            response: BankHttpAccountSummaryOutcome::Delivered { summary, .. },
        } => summary.current_balance_minor,
        other => panic!("the owner balance was not delivered: {other:?}"),
    }
}

pub(super) async fn activity_entries(
    client: &reqwest::Client,
    address: SocketAddr,
    request_id: &str,
) -> Vec<BankHttpAccountActivityItem> {
    let outcome = client
        .post(format!("http://{address}/v1/queries/account-activity/page"))
        .json(&serde_json::json!({
            "request_id": request_id,
            "controls": request_controls(16),
            "account": "fixture:100"
        }))
        .send()
        .await
        .expect("node activity snapshot should respond")
        .json::<BankUserNodeAccountActivityPageOutcome>()
        .await
        .expect("node activity snapshot should remain typed");
    match outcome {
        BankUserNodeAccountActivityPageOutcome::Forwarded {
            response:
                BankHttpAccountActivityPageOutcome::Delivered {
                    activity,
                    continuation: None,
                    ..
                },
        } => activity.entries,
        other => panic!("complete owner activity was not delivered: {other:?}"),
    }
}
