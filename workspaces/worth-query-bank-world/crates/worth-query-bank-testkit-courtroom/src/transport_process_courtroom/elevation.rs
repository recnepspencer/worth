pub use std::net::SocketAddr;

pub use bank_http_adapter::{
    BankHttpCommitDisposition, BankHttpDenialKind, BankHttpElevationApprovalOutcome,
    BankHttpElevationRequestOutcome, BankHttpElevationRevocationOutcome,
    BankHttpMandatoryReviewOutcome,
};
pub use bank_user_node::{
    BankUserNodeElevationApprovalOutcome, BankUserNodeElevationRequestOutcome,
    BankUserNodeElevationRevocationOutcome, BankUserNodeMandatoryReviewOutcome,
};

pub async fn assert_multi_actor_elevation(
    client: &reqwest::Client,
    requester: SocketAddr,
    approver: SocketAddr,
    reviewer: SocketAddr,
) {
    let requested = post::<BankUserNodeElevationRequestOutcome>(
        client,
        requester,
        "/v1/estate/elevation/request",
        serde_json::json!({
            "request_id": "process-elevation-request",
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "process-elevation-request-key",
            "estate": "fixture:3",
            "access": 31,
            "mandatory_review": 32,
            "upper_bound_grant": 20,
            "reason": "prevent_immediate_loss",
            "field": "account_details",
            "duration_seconds": 300
        }),
    )
    .await;
    let elevation = match requested {
        BankUserNodeElevationRequestOutcome::Forwarded {
            response:
                BankHttpElevationRequestOutcome::Requested {
                    disposition: BankHttpCommitDisposition::Committed,
                    elevation,
                    ..
                },
        } => elevation,
        other => panic!("requester node did not mint an opaque elevation: {other:?}"),
    };
    let self_approval = approve(client, requester, "self-approval", &elevation).await;
    assert!(matches!(
        self_approval,
        BankUserNodeElevationApprovalOutcome::Forwarded {
            response: BankHttpElevationApprovalOutcome::Denied { denial, .. }
        } if denial.kind == BankHttpDenialKind::PermissionDenied
    ));
    let approved = approve(client, approver, "process-approval", &elevation).await;
    assert!(matches!(
        approved,
        BankUserNodeElevationApprovalOutcome::Forwarded {
            response: BankHttpElevationApprovalOutcome::Approved {
                disposition: BankHttpCommitDisposition::Committed,
                ..
            }
        }
    ));
    let approval_replay = approve(client, approver, "process-approval", &elevation).await;
    assert!(matches!(
        approval_replay,
        BankUserNodeElevationApprovalOutcome::Forwarded {
            response: BankHttpElevationApprovalOutcome::Approved {
                disposition: BankHttpCommitDisposition::AlreadyCommitted,
                ..
            }
        }
    ));
    let closed = post::<BankUserNodeElevationRevocationOutcome>(
        client,
        approver,
        "/v1/estate/elevation/revoke",
        serde_json::json!({
            "request_id": "process-elevation-revoke",
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "process-revoke-key",
            "elevation": elevation
        }),
    )
    .await;
    let mandatory_review = match closed {
        BankUserNodeElevationRevocationOutcome::Forwarded {
            response:
                BankHttpElevationRevocationOutcome::Closed {
                    disposition: BankHttpCommitDisposition::Committed,
                    mandatory_review,
                    ..
                },
        } => mandatory_review,
        other => panic!("approver node did not close the elevation: {other:?}"),
    };
    let wrong_review = review(client, approver, "wrong-review", &mandatory_review).await;
    assert!(matches!(
        wrong_review,
        BankUserNodeMandatoryReviewOutcome::Forwarded {
            response: BankHttpMandatoryReviewOutcome::Denied { denial, .. }
        } if denial.kind == BankHttpDenialKind::PermissionDenied
    ));
    let reviewed = review(client, reviewer, "process-review", &mandatory_review).await;
    assert!(matches!(
        reviewed,
        BankUserNodeMandatoryReviewOutcome::Forwarded {
            response: BankHttpMandatoryReviewOutcome::Reviewed {
                disposition: BankHttpCommitDisposition::Committed,
                ..
            }
        }
    ));
    let replayed = review(client, reviewer, "process-review", &mandatory_review).await;
    assert!(matches!(
        replayed,
        BankUserNodeMandatoryReviewOutcome::Forwarded {
            response: BankHttpMandatoryReviewOutcome::Reviewed {
                disposition: BankHttpCommitDisposition::AlreadyCommitted,
                ..
            }
        }
    ));
}

pub async fn approve(
    client: &reqwest::Client,
    address: SocketAddr,
    key: &str,
    elevation: &str,
) -> BankUserNodeElevationApprovalOutcome {
    post(
        client,
        address,
        "/v1/estate/elevation/approve",
        serde_json::json!({
            "request_id": format!("approve-{key}"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": key,
            "elevation": elevation
        }),
    )
    .await
}

pub async fn review(
    client: &reqwest::Client,
    address: SocketAddr,
    key: &str,
    review: &str,
) -> BankUserNodeMandatoryReviewOutcome {
    post(
        client,
        address,
        "/v1/estate/elevation/review",
        serde_json::json!({
            "request_id": format!("review-{key}"),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": key,
            "mandatory_review": review
        }),
    )
    .await
}

pub async fn post<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    address: SocketAddr,
    path: &str,
    request: serde_json::Value,
) -> T {
    client
        .post(format!("http://{address}{path}"))
        .json(&request)
        .send()
        .await
        .expect("elevation node request should respond")
        .json()
        .await
        .expect("elevation node response should remain typed")
}
