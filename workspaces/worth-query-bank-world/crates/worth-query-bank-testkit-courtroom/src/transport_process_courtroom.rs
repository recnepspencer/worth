#[path = "transport_process_courtroom/credential_lifecycle.rs"]
pub mod credential_lifecycle;
#[path = "transport_process_courtroom/effect_observation.rs"]
pub mod effect_observation;
#[path = "transport_process_courtroom/elevation.rs"]
pub mod elevation;
#[path = "transport_process_courtroom/estate_correction.rs"]
pub mod estate_correction;
#[path = "transport_process_courtroom/identity_world.rs"]
pub mod identity_world;
#[path = "transport_process_courtroom/live.rs"]
pub mod live;
#[path = "transport_process_courtroom/payment.rs"]
pub mod payment;
#[path = "transport_process_courtroom/process.rs"]
pub mod process;
#[path = "transport_process_courtroom/rail_completion.rs"]
pub mod rail_completion;
#[path = "transport_process_courtroom/recorded_correction.rs"]
pub mod recorded_correction;
#[path = "transport_process_courtroom/recovery.rs"]
pub mod recovery;
pub use crate::facade::support;
#[path = "transport_process_courtroom/world.rs"]
pub mod world;

pub use std::net::SocketAddr;

pub use bank_http_adapter::{
    BankHttpAccountSummaryOutcome, BankHttpDenialKind, BankHttpMutationOutcome,
    BankHttpPostingPurpose,
};
pub use bank_user_node::{
    BankUserNodeAccountActivityPageOutcome, BankUserNodeAccountSummaryOutcome,
    BankUserNodeDenialKind, BankUserNodeMutationOutcome,
};
pub use effect_observation::{activity_entries, current_balance};
pub use world::TransportProcessWorld;

pub async fn post_node<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    address: SocketAddr,
    path: &str,
    request: &serde_json::Value,
) -> T {
    client
        .post(format!("http://{address}{path}"))
        .json(request)
        .send()
        .await
        .expect("node request should respond")
        .json()
        .await
        .expect("node response should be typed")
}

pub async fn assert_malformed_node_request_is_typed(client: &reqwest::Client, address: SocketAddr) {
    let outcome = client
        .post(format!("http://{address}/v1/mutations"))
        .json(&serde_json::json!({ "request_id": 42 }))
        .send()
        .await
        .expect("malformed node request should respond")
        .json::<BankUserNodeMutationOutcome>()
        .await
        .expect("malformed node response should remain typed");
    assert!(matches!(
        outcome,
        BankUserNodeMutationOutcome::Denied { denial }
            if denial.kind == BankUserNodeDenialKind::MalformedRequest
    ));
}

pub async fn assert_mutation_response_loss_and_cross_user_denial(
    client: &reqwest::Client,
    primary: SocketAddr,
    peer: SocketAddr,
) {
    let balance_before = current_balance(client, primary, "send-balance-before").await;
    let activity_before = activity_entries(client, primary, "send-activity-before").await;
    let request = serde_json::json!({
        "request_id": "process-send-response-loss",
        "command": "create", "custody_id": "process-send", "sequence": 1,
        "controls": { "deadline_milliseconds": 5_000, "maximum_history_work": 128 },
        "idempotency_key": "process-send-response-loss-key",
        "operation": { "operation": "send_money", "from": "fixture:100", "recipient": "fixture:2", "amount_minor_units": 25 }
    });
    drop(
        client
            .post(format!("http://{primary}/v1/mutations"))
            .json(&request)
            .send()
            .await
            .expect("mutation response should be droppable after headers"),
    );
    let balance_after = current_balance(client, primary, "send-balance-after").await;
    let activity_after = activity_entries(client, primary, "send-activity-after").await;
    assert_eq!(balance_after, balance_before - 25);
    assert_eq!(activity_after.len(), activity_before.len() + 1);
    assert_eq!(
        activity_after
            .iter()
            .filter(|entry| entry.purpose == BankHttpPostingPurpose::Transfer
                && entry.amount_minor == -25)
            .count(),
        1,
        "the lost response must follow exactly one real transfer posting"
    );
    let resume = serde_json::json!({ "request_id": "process-send-resume", "command": "resume", "custody_id": "process-send", "sequence": 2,
        "controls": { "deadline_milliseconds": 5000, "maximum_history_work": 128 } });
    let crossed = node_mutation(client, peer, &resume).await;
    assert!(
        matches!(crossed, BankUserNodeMutationOutcome::Forwarded { response: BankHttpMutationOutcome::Refused { denial, .. } } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    let replayed = node_mutation(client, primary, &resume).await;
    assert!(matches!(
        replayed,
        BankUserNodeMutationOutcome::Forwarded {
            response: BankHttpMutationOutcome::Completed {
                recovered: false,
                ..
            }
        }
    ));
    assert_eq!(
        current_balance(client, primary, "send-balance-retry").await,
        balance_after
    );
    assert_eq!(
        activity_entries(client, primary, "send-activity-retry").await,
        activity_after
    );
    let closed = node_mutation(client, primary, &serde_json::json!({ "request_id": "process-send-close", "command": "close", "custody_id": "process-send", "sequence": 3,
        "controls": { "deadline_milliseconds": 5000,  } })).await;
    assert!(matches!(
        closed,
        BankUserNodeMutationOutcome::Forwarded {
            response: BankHttpMutationOutcome::Closed { .. }
        }
    ));
}

pub async fn node_mutation(
    client: &reqwest::Client,
    address: SocketAddr,
    request: &serde_json::Value,
) -> BankUserNodeMutationOutcome {
    client
        .post(format!("http://{address}/v1/mutations"))
        .json(request)
        .send()
        .await
        .expect("node mutation should respond")
        .json()
        .await
        .expect("node mutation response should be typed")
}

pub async fn assert_lawful_summary(client: &reqwest::Client, address: SocketAddr) {
    let outcome = account_summary(client, address, "fixture:100").await;
    match outcome {
        BankUserNodeAccountSummaryOutcome::Forwarded {
            response: BankHttpAccountSummaryOutcome::Delivered { summary, .. },
        } => assert_eq!(summary.account, "fixture:100"),
        other => panic!("lawful process query did not deliver: {other:?}"),
    }
}

pub async fn assert_node_request_saturation(client: &reqwest::Client, address: SocketAddr) {
    let mut requests = tokio::task::JoinSet::new();
    for _ in 0..16 {
        let client = client.clone();
        requests.spawn(async move { account_summary(&client, address, "fixture:100").await });
    }
    let mut forwarded = 0;
    let mut saturated = 0;
    while let Some(outcome) = requests.join_next().await {
        match outcome.expect("concurrent node request should join") {
            BankUserNodeAccountSummaryOutcome::Forwarded {
                response: BankHttpAccountSummaryOutcome::Delivered { .. },
            } => forwarded += 1,
            BankUserNodeAccountSummaryOutcome::Denied { denial }
                if denial.kind == BankUserNodeDenialKind::RequestSaturated =>
            {
                saturated += 1;
            }
            other => panic!("request-concurrency court returned an unrelated posture: {other:?}"),
        }
    }
    assert!(forwarded >= 1, "one admitted request must cross the node");
    assert!(
        saturated >= 1,
        "the one-request node must reject excess work"
    );
    assert_lawful_summary(client, address).await;
}

pub async fn assert_cross_user_denial(client: &reqwest::Client, address: SocketAddr) {
    let outcome = account_summary(client, address, "fixture:100").await;
    match outcome {
        BankUserNodeAccountSummaryOutcome::Forwarded {
            response: BankHttpAccountSummaryOutcome::Denied { denial, .. },
        } => assert_eq!(denial.kind, BankHttpDenialKind::PermissionDenied),
        other => panic!("cross-user account query opened authority: {other:?}"),
    }
}

pub async fn activity_page(
    client: &reqwest::Client,
    address: SocketAddr,
    request_id: &str,
) -> BankUserNodeAccountActivityPageOutcome {
    client
        .post(format!("http://{address}/v1/queries/account-activity/page"))
        .json(&serde_json::json!({
            "request_id": request_id,
            "controls": request_controls(2),
            "account": "fixture:100"
        }))
        .send()
        .await
        .expect("node activity page should respond")
        .json()
        .await
        .expect("node activity page should be typed")
}

pub async fn activity_resume(
    client: &reqwest::Client,
    address: SocketAddr,
    request_id: &str,
    continuation: &str,
) -> BankUserNodeAccountActivityPageOutcome {
    client
        .post(format!(
            "http://{address}/v1/queries/account-activity/resume"
        ))
        .json(&serde_json::json!({
            "request_id": request_id,
            "controls": request_controls(2),
            "account": "fixture:100",
            "continuation": continuation
        }))
        .send()
        .await
        .expect("node activity resume should respond")
        .json()
        .await
        .expect("node activity resume should be typed")
}

pub fn activity_token(outcome: &BankUserNodeAccountActivityPageOutcome) -> &str {
    match outcome {
        BankUserNodeAccountActivityPageOutcome::Forwarded {
            response:
                bank_http_adapter::BankHttpAccountActivityPageOutcome::Delivered {
                    continuation: Some(token),
                    ..
                },
        } => token,
        other => panic!("first process page did not continue: {other:?}"),
    }
}

pub async fn assert_peer_cannot_resume(client: &reqwest::Client, peer: SocketAddr, token: &str) {
    let outcome = activity_resume(client, peer, "peer-token-substitution", token).await;
    assert!(matches!(
        outcome,
        BankUserNodeAccountActivityPageOutcome::Forwarded {
            response: bank_http_adapter::BankHttpAccountActivityPageOutcome::Denied {
                denial,
                ..
            },
        } if denial.kind == BankHttpDenialKind::Stale
    ));
}

pub fn assert_terminal_resume(outcome: &BankUserNodeAccountActivityPageOutcome) {
    assert!(matches!(
        outcome,
        BankUserNodeAccountActivityPageOutcome::Forwarded {
            response: bank_http_adapter::BankHttpAccountActivityPageOutcome::Delivered {
                continuation: None,
                ..
            },
        }
    ));
}

pub fn request_controls(maximum_results: usize) -> serde_json::Value {
    serde_json::json!(bank_http_adapter::BankHttpRequestControls::new(
        5_000,
        maximum_results,
    ))
}

pub async fn assert_restarted_node_has_no_session(client: &reqwest::Client, address: SocketAddr) {
    let outcome = account_summary(client, address, "fixture:100").await;
    match outcome {
        BankUserNodeAccountSummaryOutcome::Denied { denial } => {
            assert_eq!(denial.kind, BankUserNodeDenialKind::NoAuthenticatedSession)
        }
        other => panic!("restarted node retained stale session authority: {other:?}"),
    }
}

pub async fn account_summary(
    client: &reqwest::Client,
    address: SocketAddr,
    account: &str,
) -> BankUserNodeAccountSummaryOutcome {
    account_summary_with_request_id(client, address, account, "process-summary").await
}

pub async fn account_summary_with_request_id(
    client: &reqwest::Client,
    address: SocketAddr,
    account: &str,
    request_id: &str,
) -> BankUserNodeAccountSummaryOutcome {
    client
        .post(format!("http://{address}/v1/queries/account-summary"))
        .json(&serde_json::json!({
            "request_id": request_id,
            "controls": request_controls(1),
            "account": account
        }))
        .send()
        .await
        .expect("node query should respond")
        .json()
        .await
        .expect("node query response should be typed")
}
