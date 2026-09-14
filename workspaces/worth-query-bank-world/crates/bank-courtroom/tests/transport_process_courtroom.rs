#[path = "transport_process_courtroom/credential_lifecycle.rs"]
mod credential_lifecycle;
#[path = "transport_process_courtroom/elevation.rs"]
mod elevation;
#[path = "transport_process_courtroom/identity_world.rs"]
mod identity_world;
#[path = "transport_process_courtroom/live.rs"]
mod live;
#[path = "transport_process_courtroom/process.rs"]
mod process;
#[path = "transport_process_courtroom/recovery.rs"]
mod recovery;
#[path = "transport_process_courtroom/world.rs"]
mod world;

use std::net::SocketAddr;

use bank_http_adapter::{
    BankHttpAccountSummaryOutcome, BankHttpCommitDisposition, BankHttpDenialKind,
    BankHttpMutationOutcome,
};
use bank_user_node::{
    BankUserNodeAccountActivityPageOutcome, BankUserNodeAccountSummaryOutcome,
    BankUserNodeDenialKind, BankUserNodeMutationOutcome,
};
use world::TransportProcessWorld;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn independently_authenticated_nodes_cross_real_process_and_tcp_boundaries() {
    let mut world = TransportProcessWorld::start().await;
    world.authenticate_participants().await;
    elevation::assert_multi_actor_elevation(
        &world.client,
        world.peer_address,
        world.approver_address,
        world.reviewer_address,
    )
    .await;
    assert_malformed_node_request_is_typed(&world.client, world.primary_address).await;
    assert_lawful_summary(&world.client, world.primary_address).await;
    assert_node_request_saturation(&world.client, world.primary_address).await;
    assert_cross_user_denial(&world.client, world.peer_address).await;
    assert_mutation_response_loss_and_cross_user_denial(
        &world.client,
        world.primary_address,
        world.peer_address,
    )
    .await;
    live::assert_live_stream_lifecycle_and_revocation(&world.client, world.primary_address).await;
    world.authenticate_primary().await;
    recovery::assert_opaque_recovery_is_owned_by_the_authenticated_specialist(
        &world.client,
        world.primary_address,
        world.peer_address,
    )
    .await;
    let first = activity_page(&world.client, world.primary_address, "process-page-1").await;
    let token = activity_token(&first).to_owned();
    assert_eq!(
        activity_page(&world.client, world.primary_address, "process-page-1").await,
        first,
        "a lost first-page response must replay exactly"
    );
    assert_peer_cannot_resume(&world.client, world.peer_address, &token).await;

    let restarted_address = world.crash_and_restart_primary().await;
    assert_restarted_node_has_no_session(&world.client, restarted_address).await;
    world.authenticate_primary().await;
    let resumed =
        activity_resume(&world.client, restarted_address, "process-resume-1", &token).await;
    assert_terminal_resume(&resumed);
    assert_eq!(
        activity_resume(&world.client, restarted_address, "process-resume-1", &token,).await,
        resumed,
        "a lost resume response must replay exactly"
    );
    credential_lifecycle::assert_expired_session_fails_closed(&world).await;
    world.shutdown().await;
}

async fn post_node<T: serde::de::DeserializeOwned>(
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

async fn assert_malformed_node_request_is_typed(client: &reqwest::Client, address: SocketAddr) {
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

async fn assert_mutation_response_loss_and_cross_user_denial(
    client: &reqwest::Client,
    primary: SocketAddr,
    peer: SocketAddr,
) {
    let request = serde_json::json!({
        "request_id": "process-send-response-loss",
        "controls": { "deadline_milliseconds": 5_000 },
        "idempotency_key": "process-send-response-loss-key",
        "operation": "send_money",
        "from": "fixture:100",
        "recipient": "fixture:2",
        "amount_minor_units": 25
    });
    drop(
        client
            .post(format!("http://{primary}/v1/mutations"))
            .json(&request)
            .send()
            .await
            .expect("mutation response should be droppable after headers"),
    );
    let replayed = node_mutation(client, primary, &request).await;
    assert!(matches!(
        replayed,
        BankUserNodeMutationOutcome::Forwarded {
            response: BankHttpMutationOutcome::Applied {
                disposition: BankHttpCommitDisposition::AlreadyCommitted,
                ..
            }
        }
    ));
    let crossed = node_mutation(client, peer, &request).await;
    assert!(matches!(
        crossed,
        BankUserNodeMutationOutcome::Forwarded {
            response: BankHttpMutationOutcome::NotApplied { denial, .. }
        } if denial.kind == BankHttpDenialKind::PermissionDenied
    ));
}

async fn node_mutation(
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

async fn assert_lawful_summary(client: &reqwest::Client, address: SocketAddr) {
    let outcome = account_summary(client, address, "fixture:100").await;
    match outcome {
        BankUserNodeAccountSummaryOutcome::Forwarded {
            response: BankHttpAccountSummaryOutcome::Delivered { summary, .. },
        } => assert_eq!(summary.account, "fixture:100"),
        other => panic!("lawful process query did not deliver: {other:?}"),
    }
}

async fn assert_node_request_saturation(client: &reqwest::Client, address: SocketAddr) {
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

async fn assert_cross_user_denial(client: &reqwest::Client, address: SocketAddr) {
    let outcome = account_summary(client, address, "fixture:100").await;
    match outcome {
        BankUserNodeAccountSummaryOutcome::Forwarded {
            response: BankHttpAccountSummaryOutcome::Denied { denial, .. },
        } => assert_eq!(denial.kind, BankHttpDenialKind::PermissionDenied),
        other => panic!("cross-user account query opened authority: {other:?}"),
    }
}

async fn activity_page(
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

async fn activity_resume(
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

fn activity_token(outcome: &BankUserNodeAccountActivityPageOutcome) -> &str {
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

async fn assert_peer_cannot_resume(client: &reqwest::Client, peer: SocketAddr, token: &str) {
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

fn assert_terminal_resume(outcome: &BankUserNodeAccountActivityPageOutcome) {
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

fn request_controls(maximum_results: usize) -> serde_json::Value {
    serde_json::json!({
        "deadline_milliseconds": 5_000,
        "maximum_results": maximum_results,
        "maximum_work": 20_000
    })
}

async fn assert_restarted_node_has_no_session(client: &reqwest::Client, address: SocketAddr) {
    let outcome = account_summary(client, address, "fixture:100").await;
    match outcome {
        BankUserNodeAccountSummaryOutcome::Denied { denial } => {
            assert_eq!(denial.kind, BankUserNodeDenialKind::NoAuthenticatedSession)
        }
        other => panic!("restarted node retained stale session authority: {other:?}"),
    }
}

async fn account_summary(
    client: &reqwest::Client,
    address: SocketAddr,
    account: &str,
) -> BankUserNodeAccountSummaryOutcome {
    client
        .post(format!("http://{address}/v1/queries/account-summary"))
        .json(&serde_json::json!({
            "request_id": "process-summary",
            "controls": {
                "deadline_milliseconds": 5_000,
                "maximum_results": 1,
                "maximum_work": 20_000
            },
            "account": account
        }))
        .send()
        .await
        .expect("node query should respond")
        .json()
        .await
        .expect("node query response should be typed")
}
