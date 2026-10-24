use std::net::SocketAddr;
use std::time::Duration;

use bank_http_adapter::BankHttpMutationOutcome;
use bank_user_node::{
    BankUserNodeAuthorizationOutcome, BankUserNodeDenial, BankUserNodeDenialKind,
    BankUserNodeMutationOutcome,
};

use super::node_mutation;

pub async fn assert_live_stream_lifecycle_and_revocation(
    client: &reqwest::Client,
    node: SocketAddr,
) {
    let mut stream = open_stream(client, node, "process-live", 5_000).await;
    assert_eq!(stream.status(), reqwest::StatusCode::OK);
    let mut transcript = String::new();
    read_until(&mut stream, &mut transcript, "\"event\":\"opened\"").await;
    assert_saturated(client, node).await;
    assert_update_crosses_processes(client, node, &mut stream, &mut transcript).await;

    drop(stream);
    let mut deadline_stream = open_after_disconnect(client, node).await;
    assert_eq!(deadline_stream.status(), reqwest::StatusCode::OK);
    read_until(
        &mut deadline_stream,
        &mut String::new(),
        "deadline_exceeded",
    )
    .await;
    drop(deadline_stream);

    let mut revoked_stream = open_stream(client, node, "process-live-revoked", 5_000).await;
    assert_eq!(revoked_stream.status(), reqwest::StatusCode::OK);
    let mut revoked_transcript = String::new();
    read_until(
        &mut revoked_stream,
        &mut revoked_transcript,
        "\"event\":\"opened\"",
    )
    .await;
    assert_eq!(
        revoke_session(client, node).await,
        BankUserNodeAuthorizationOutcome::Revoked
    );
    read_until(&mut revoked_stream, &mut revoked_transcript, "cancelled").await;
}

async fn open_after_disconnect(client: &reqwest::Client, node: SocketAddr) -> reqwest::Response {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let response = open_stream(client, node, "process-live-deadline", 250).await;
        if response.status() == reqwest::StatusCode::OK {
            return response;
        }
        assert_eq!(
            response.status(),
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            "disconnect cleanup may transiently retain only the existing live permit"
        );
        assert!(
            tokio::time::Instant::now() < deadline,
            "dropped SSE response did not release its node and server permits"
        );
        drop(response);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

pub(super) async fn revoke_session(
    client: &reqwest::Client,
    node: SocketAddr,
) -> BankUserNodeAuthorizationOutcome {
    client
        .post(format!("http://{node}/session/revoke"))
        .send()
        .await
        .expect("session revocation should respond")
        .json()
        .await
        .expect("session revocation should remain typed")
}

async fn assert_saturated(client: &reqwest::Client, node: SocketAddr) {
    let response = open_stream(client, node, "process-live-saturated", 5_000).await;
    assert_eq!(response.status(), reqwest::StatusCode::TOO_MANY_REQUESTS);
    let denial = response
        .json::<BankUserNodeDenial>()
        .await
        .expect("stream saturation should remain typed");
    assert_eq!(denial.kind, BankUserNodeDenialKind::RequestSaturated);
}

async fn assert_update_crosses_processes(
    client: &reqwest::Client,
    node: SocketAddr,
    stream: &mut reqwest::Response,
    transcript: &mut String,
) {
    let mutation = node_mutation(
        client,
        node,
        &serde_json::json!({
            "request_id": "process-live-send",
            "command": "create", "custody_id": "process-live-send", "sequence": 1,
            "controls": { "deadline_milliseconds": 5_000, "maximum_history_work": 128 },
            "idempotency_key": "process-live-send-key",
            "operation": { "operation": "send_money",
            "from": "fixture:100", "recipient": "fixture:2", "amount_minor_units": 1 }
        }),
    )
    .await;
    assert!(matches!(
        mutation,
        BankUserNodeMutationOutcome::Forwarded {
            response: BankHttpMutationOutcome::Completed {
                recovered: false,
                ..
            }
        }
    ));
    let closed = node_mutation(client, node, &serde_json::json!({
        "request_id": "process-live-close", "command": "close", "custody_id": "process-live-send", "sequence": 2,
        "controls": { "deadline_milliseconds": 5000,  }
    })).await;
    assert!(matches!(
        closed,
        BankUserNodeMutationOutcome::Forwarded {
            response: BankHttpMutationOutcome::Closed { .. }
        }
    ));
    read_until(stream, transcript, "\"event\":\"update\"").await;
    assert!(transcript.contains("\"capability_purpose\":\"account_activity_review\""));
    assert!(transcript.contains("\"omission\":\"no_omission\""));
}

pub(super) async fn open_stream(
    client: &reqwest::Client,
    node: SocketAddr,
    request_id: &str,
    deadline_milliseconds: u64,
) -> reqwest::Response {
    client
        .post(format!("http://{node}/v1/live/account-activity"))
        .json(&serde_json::json!({
            "request_id": request_id,
            "controls": bank_http_adapter::BankHttpRequestControls::new(deadline_milliseconds, 8),
            "account": "fixture:100",
            "source_buffer_capacity": 1
        }))
        .send()
        .await
        .expect("process SSE request should respond")
}

pub(super) async fn read_until(
    response: &mut reqwest::Response,
    transcript: &mut String,
    marker: &str,
) {
    tokio::time::timeout(Duration::from_secs(8), async {
        while !transcript.contains(marker) {
            let chunk = response
                .chunk()
                .await
                .expect("process SSE chunk should read")
                .unwrap_or_else(|| panic!("process SSE closed before {marker}: {transcript}"));
            transcript.push_str(std::str::from_utf8(&chunk).expect("SSE must be UTF-8"));
        }
    })
    .await
    .unwrap_or_else(|_| panic!("process SSE did not publish {marker}: {transcript}"));
}
