use std::sync::Arc;
use std::time::Duration;

use bank_domain::model::AccountId;

use super::super::super::protocol::{
    BankHttpCommitDisposition, BankHttpDenialKind, BankHttpElevationApprovalOutcome,
    BankHttpElevationRequestOutcome, BankHttpElevationRevocationOutcome,
    BankHttpMandatoryReviewOutcome,
};
use super::super::{bind_application, BankHttpServerConfiguration};
use super::fixture::application;

#[tokio::test]
async fn multi_actor_elevation_retains_exact_phase_across_wrong_actor_and_replay() {
    let server = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let origin = format!("http://{}", server.local_address());
    let client = reqwest::Client::new();

    let requested = client
        .post(format!("{origin}/v1/estate/elevation/request"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "elevation-request-1",
            "credential": credential("test-only"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "elevation-request-key",
            "estate": "fixture:3",
            "access": 31,
            "mandatory_review": 32,
            "upper_bound_grant": 20,
            "reason": "prevent_immediate_loss",
            "field": "account_details",
            "duration_seconds": 300
        }))
        .send()
        .await
        .expect("request should cross TCP")
        .json::<BankHttpElevationRequestOutcome>()
        .await
        .expect("request outcome should decode");
    let elevation = match requested {
        BankHttpElevationRequestOutcome::Requested {
            disposition: BankHttpCommitDisposition::Committed,
            elevation,
            ..
        } => elevation,
        other => panic!("elevation should be requested: {other:?}"),
    };

    let self_approval = approve(&client, &origin, "test-only", "self-key", &elevation).await;
    assert!(matches!(
        self_approval,
        BankHttpElevationApprovalOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::PermissionDenied
    ));

    let raw_axis = client
        .post(format!("{origin}/v1/estate/elevation/approve"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "raw-axis-approval",
            "credential": credential("approver"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "raw-axis-key",
            "elevation": elevation,
            "estate": "fixture:999"
        }))
        .send()
        .await
        .expect("raw-axis request should respond")
        .json::<BankHttpElevationApprovalOutcome>()
        .await
        .expect("raw-axis response should decode");
    assert!(matches!(
        raw_axis,
        BankHttpElevationApprovalOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::MalformedRequest
    ));

    let wrong_phase = client
        .post(format!("{origin}/v1/estate/elevation/revoke"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "revoke-requested-phase",
            "credential": credential("approver"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "wrong-phase-key",
            "elevation": elevation
        }))
        .send()
        .await
        .expect("wrong-phase request should respond")
        .json::<BankHttpElevationRevocationOutcome>()
        .await
        .expect("wrong-phase response should decode");
    assert!(matches!(
        wrong_phase,
        BankHttpElevationRevocationOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::Stale
    ));

    let approval = approve(&client, &origin, "approver", "approval-key", &elevation).await;
    assert!(matches!(
        approval,
        BankHttpElevationApprovalOutcome::Approved {
            disposition: BankHttpCommitDisposition::Committed,
            ..
        }
    ));
    let approval_replay = approve(&client, &origin, "approver", "approval-key", &elevation).await;
    assert!(matches!(
        approval_replay,
        BankHttpElevationApprovalOutcome::Approved {
            disposition: BankHttpCommitDisposition::AlreadyCommitted,
            ..
        }
    ));

    let closed = client
        .post(format!("{origin}/v1/estate/elevation/revoke"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "elevation-revoke-1",
            "credential": credential("approver"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "revoke-key",
            "elevation": elevation
        }))
        .send()
        .await
        .expect("revoke should cross TCP")
        .json::<BankHttpElevationRevocationOutcome>()
        .await
        .expect("revoke outcome should decode");
    let review = match closed {
        BankHttpElevationRevocationOutcome::Closed {
            disposition: BankHttpCommitDisposition::Committed,
            mandatory_review,
            ..
        } => mandatory_review,
        other => panic!("elevation should close: {other:?}"),
    };

    let wrong_review = complete_review(&client, &origin, "approver", "wrong-review", &review).await;
    assert!(matches!(
        wrong_review,
        BankHttpMandatoryReviewOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::PermissionDenied
    ));
    let reviewed = complete_review(&client, &origin, "reviewer", "review-key", &review).await;
    assert!(matches!(
        reviewed,
        BankHttpMandatoryReviewOutcome::Reviewed {
            disposition: BankHttpCommitDisposition::Committed,
            ..
        }
    ));
    let replay = complete_review(&client, &origin, "reviewer", "review-key", &review).await;
    assert!(matches!(
        replay,
        BankHttpMandatoryReviewOutcome::Reviewed {
            disposition: BankHttpCommitDisposition::AlreadyCommitted,
            ..
        }
    ));
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn expired_elevation_token_opens_no_approval_phase() {
    let server = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral()
            .with_opaque_handle_lifetime(std::time::Duration::from_millis(1)),
    )
    .await
    .expect("HTTP server should bind");
    let origin = format!("http://{}", server.local_address());
    let client = reqwest::Client::new();
    let requested = client
        .post(format!("{origin}/v1/estate/elevation/request"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "expiring-elevation",
            "credential": credential("test-only"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "expiring-elevation-key",
            "estate": "fixture:3",
            "access": 41,
            "mandatory_review": 42,
            "upper_bound_grant": 20,
            "reason": "prevent_immediate_loss",
            "field": "account_details",
            "duration_seconds": 300
        }))
        .send()
        .await
        .expect("request should respond")
        .json::<BankHttpElevationRequestOutcome>()
        .await
        .expect("request should decode");
    let BankHttpElevationRequestOutcome::Requested { elevation, .. } = requested else {
        panic!("elevation request should commit: {requested:?}");
    };
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let approval = approve(&client, &origin, "approver", "expired-approval", &elevation).await;
    assert!(matches!(
        approval,
        BankHttpElevationApprovalOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::Stale
    ));
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn lost_approval_response_replays_the_committed_transition() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let server = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let address = server.local_address();
    let origin = format!("http://{address}");
    let client = reqwest::Client::new();
    let requested = client
        .post(format!("{origin}/v1/estate/elevation/request"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "approval-response-loss-request",
            "credential": credential("test-only"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "approval-response-loss-request-key",
            "estate": "fixture:3",
            "access": 51,
            "mandatory_review": 52,
            "upper_bound_grant": 20,
            "reason": "prevent_immediate_loss",
            "field": "account_details",
            "duration_seconds": 300
        }))
        .send()
        .await
        .expect("request should cross TCP")
        .json::<BankHttpElevationRequestOutcome>()
        .await
        .expect("request outcome should decode");
    let BankHttpElevationRequestOutcome::Requested { elevation, .. } = requested else {
        panic!("the initial elevation must commit: {requested:?}");
    };

    let body = serde_json::json!({
        "protocol": "v1",
        "request_id": "approval-response-loss",
        "credential": credential("approver"),
        "controls": { "deadline_milliseconds": 5_000 },
        "idempotency_key": "approval-response-loss-key",
        "elevation": elevation,
    })
    .to_string();
    let frame = format!(
        "POST /v1/estate/elevation/approve HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let mut disconnected = tokio::net::TcpStream::connect(address)
        .await
        .expect("approval socket should connect");
    disconnected
        .write_all(frame.as_bytes())
        .await
        .expect("complete approval request should reach TCP");
    let mut response_started = [0_u8; 1];
    disconnected
        .read_exact(&mut response_started)
        .await
        .expect("the server should start its response after approval commits");
    drop(disconnected);

    let replay = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let outcome = approve(
                &client,
                &origin,
                "approver",
                "approval-response-loss-key",
                &elevation,
            )
            .await;
            match outcome {
                BankHttpElevationApprovalOutcome::Denied { denial, .. }
                    if denial.kind == BankHttpDenialKind::Stale =>
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                other => break other,
            }
        }
    })
    .await
    .expect("the first approval should settle within the endpoint deadline");
    assert!(
        matches!(
            replay,
            BankHttpElevationApprovalOutcome::Approved {
                disposition: BankHttpCommitDisposition::AlreadyCommitted,
                ..
            }
        ),
        "the lost response must not execute approval again: {replay:?}"
    );
    server.shutdown().await.expect("server should shut down");
}

async fn approve(
    client: &reqwest::Client,
    origin: &str,
    actor: &str,
    key: &str,
    elevation: &str,
) -> BankHttpElevationApprovalOutcome {
    client
        .post(format!("{origin}/v1/estate/elevation/approve"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": format!("approve-{key}"),
            "credential": credential(actor),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": key,
            "elevation": elevation
        }))
        .send()
        .await
        .expect("approval should cross TCP")
        .json()
        .await
        .expect("approval outcome should decode")
}

async fn complete_review(
    client: &reqwest::Client,
    origin: &str,
    actor: &str,
    key: &str,
    review: &str,
) -> BankHttpMandatoryReviewOutcome {
    client
        .post(format!("{origin}/v1/estate/elevation/review"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": format!("review-{key}"),
            "credential": credential(actor),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": key,
            "mandatory_review": review
        }))
        .send()
        .await
        .expect("review should cross TCP")
        .json()
        .await
        .expect("review outcome should decode")
}

fn credential(actor: &str) -> serde_json::Value {
    serde_json::json!({
        "id_token": "test-only",
        "access_token": actor,
        "nonce": "test-only"
    })
}
