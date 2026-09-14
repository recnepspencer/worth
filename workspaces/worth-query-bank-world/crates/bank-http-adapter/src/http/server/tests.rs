use std::sync::Arc;
use std::time::Duration;

use bank_domain::model::AccountId;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use super::super::protocol::{
    BankHttpAccountActivityPageOutcome, BankHttpDenialKind, BankHttpQueryCapabilityPurpose,
};
use super::{bind_application, BankHttpServerConfiguration};

mod elevation;
mod fixture;
mod mutation;
mod protocol;
mod recovery;

use fixture::{application, CausalHttpApplication};

#[tokio::test]
async fn account_activity_sse_preserves_open_and_deadline_postures() {
    let account = AccountId::new(100).unwrap();
    let application = Arc::new(application(account));
    assert_live_fixture_admits(application.as_ref(), account).await;
    let server = bind_application(
        application,
        BankHttpServerConfiguration::local_ephemeral()
            .with_maximum_live_streams(std::num::NonZeroUsize::new(1).unwrap()),
    )
    .await
    .expect("HTTP server should bind");
    let endpoint = format!("http://{}/v1/live/account-activity", server.local_address());
    let client = reqwest::Client::new();
    let mut response = client
        .post(endpoint)
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "activity-stream-1",
            "credential": {
                "id_token": "test-only",
                "access_token": "test-only",
                "nonce": "test-only"
            },
            "controls": {
                "deadline_milliseconds": 3_000,
                "maximum_results": 8,
                "maximum_work": 2_048
            },
            "account": account.canonical_text(),
            "source_buffer_capacity": 16
        }))
        .send()
        .await
        .expect("SSE request should connect");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let saturated = client
        .post(format!(
            "http://{}/v1/live/account-activity",
            server.local_address()
        ))
        .json(&live_request(account, "activity-stream-saturated", 1_000))
        .send()
        .await
        .expect("second SSE request should receive a response");
    assert_eq!(saturated.status(), reqwest::StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        saturated
            .json::<super::super::protocol::BankHttpDenial>()
            .await
            .expect("saturation response should be typed")
            .kind,
        BankHttpDenialKind::Saturated
    );
    let mut transcript = String::new();
    tokio::time::timeout(
        Duration::from_secs(1),
        read_sse_until(&mut response, &mut transcript, "\"event\":\"opened\""),
    )
    .await
    .expect("SSE open should arrive promptly");
    let mutation = client
        .post(format!("http://{}/v1/mutations", server.local_address()))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "live-publication-deposit",
            "credential": credential_json(),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "live-publication-deposit-key",
            "operation": "deposit",
            "institution": "fixture:1",
            "account": account.canonical_text(),
            "amount_minor_units": 1
        }))
        .send()
        .await
        .expect("live-triggering mutation should complete");
    assert_eq!(mutation.status(), reqwest::StatusCode::OK);
    tokio::time::timeout(
        Duration::from_secs(2),
        read_sse_until(&mut response, &mut transcript, "\"event\":\"update\""),
    )
    .await
    .expect("SSE update should arrive before the deadline");
    tokio::time::timeout(
        Duration::from_secs(5),
        read_sse_until(&mut response, &mut transcript, "deadline_exceeded"),
    )
    .await
    .expect("SSE deadline should arrive promptly");
    assert!(transcript.contains("bank_account_activity"));
    assert!(transcript.contains("\"event\":\"opened\""));
    assert!(transcript.contains("\"capability_purpose\":\"account_activity_review\""));
    assert!(transcript.contains("\"omission\":\"no_omission\""));
    assert!(transcript.contains("\"event\":\"deadline_exceeded\""));
    drop(response);
    tokio::time::sleep(Duration::from_millis(50)).await;
    let replacement = client
        .post(format!(
            "http://{}/v1/live/account-activity",
            server.local_address()
        ))
        .json(&live_request(account, "activity-stream-replacement", 250))
        .send()
        .await
        .expect("replacement SSE request should connect");
    assert_eq!(replacement.status(), reqwest::StatusCode::OK);
    drop(replacement);
    server.shutdown().await.expect("server should shut down");
}

async fn read_sse_until(response: &mut reqwest::Response, transcript: &mut String, marker: &str) {
    while !transcript.contains(marker) {
        let chunk = response.chunk().await.expect("SSE chunk should read");
        let Some(chunk) = chunk else {
            panic!("SSE closed before {marker}: {transcript}");
        };
        transcript.push_str(std::str::from_utf8(&chunk).expect("SSE must be UTF-8"));
    }
}

#[tokio::test]
async fn opaque_continuation_replays_lost_responses_without_reusing_query_authority() {
    let account = AccountId::new(100).unwrap();
    let application = Arc::new(application(account));
    let server = bind_application(application, BankHttpServerConfiguration::local_ephemeral())
        .await
        .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let page_endpoint = format!(
        "http://{}/v1/queries/account-activity/page",
        server.local_address()
    );
    let resume_endpoint = format!(
        "http://{}/v1/queries/account-activity/resume",
        server.local_address()
    );
    let first_request = page_request(account, "activity-page-1");
    let first = post_page(&client, &page_endpoint, &first_request).await;
    let first_publication = match &first {
        BankHttpAccountActivityPageOutcome::Delivered { publication, .. } => publication,
        other => panic!("activity page did not publish: {other:?}"),
    };
    assert_eq!(first_publication.query_identity.len(), 64);
    assert_eq!(
        first_publication.capability_purpose,
        BankHttpQueryCapabilityPurpose::AccountActivityReview
    );
    let token = continuation(&first).to_owned();
    let replayed = post_page(&client, &page_endpoint, &first_request).await;
    assert_eq!(replayed, first, "initial response loss must not advance");

    let resume_request = serde_json::json!({
        "protocol": "v1",
        "request_id": "activity-resume-1",
        "credential": credential_json(),
        "controls": controls_json(1),
        "account": account.canonical_text(),
        "continuation": token
    });
    let resumed = post_page(&client, &resume_endpoint, &resume_request).await;
    let resumed_again = post_page(&client, &resume_endpoint, &resume_request).await;
    assert_eq!(
        resumed_again, resumed,
        "resume response loss must replay exactly"
    );
    assert!(matches!(
        resumed,
        BankHttpAccountActivityPageOutcome::Delivered {
            continuation: None,
            ..
        }
    ));

    let crossed_request = serde_json::json!({
        "protocol": "v1",
        "request_id": "activity-resume-crossed",
        "credential": credential_json(),
        "controls": controls_json(1),
        "account": account.canonical_text(),
        "continuation": continuation(&first)
    });
    let crossed = post_page(&client, &resume_endpoint, &crossed_request).await;
    assert!(matches!(
        crossed,
        BankHttpAccountActivityPageOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::Stale
    ));
    server.shutdown().await.expect("server should shut down");
}

async fn assert_live_fixture_admits(application: &CausalHttpApplication, account: AccountId) {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;
    use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        std::time::Instant::now() + Duration::from_secs(5),
        cancellation.token(),
    );
    let principal = application.authenticate(&scope).await;
    let controls = WorthQueryApplicationLiveControls::bounded(scope, 16, 8, 2_048).unwrap();
    match application
        .runtime
        .account_activity(account)
        .as_principal(&principal)
        .subscribe(controls)
    {
        Ok(lease) => {
            let _ = lease.close();
        }
        Err(error) => panic!("live fixture must admit: {error:?}"),
    };
}

fn page_request(account: AccountId, request_id: &str) -> serde_json::Value {
    serde_json::json!({
        "protocol": "v1",
        "request_id": request_id,
        "credential": credential_json(),
        "controls": controls_json(1),
        "account": account.canonical_text()
    })
}

fn credential_json() -> serde_json::Value {
    serde_json::json!({
        "id_token": "test-only",
        "access_token": "test-only",
        "nonce": "test-only"
    })
}

fn controls_json(maximum_results: usize) -> serde_json::Value {
    serde_json::json!({
        "deadline_milliseconds": 5_000,
        "maximum_results": maximum_results,
        "maximum_work": 20_000
    })
}

fn live_request(
    account: AccountId,
    request_id: &str,
    deadline_milliseconds: u64,
) -> serde_json::Value {
    serde_json::json!({
        "protocol": "v1",
        "request_id": request_id,
        "credential": credential_json(),
        "controls": {
            "deadline_milliseconds": deadline_milliseconds,
            "maximum_results": 8,
            "maximum_work": 2_048
        },
        "account": account.canonical_text(),
        "source_buffer_capacity": 16
    })
}

async fn post_page(
    client: &reqwest::Client,
    endpoint: &str,
    request: &serde_json::Value,
) -> BankHttpAccountActivityPageOutcome {
    client
        .post(endpoint)
        .json(request)
        .send()
        .await
        .expect("activity page request should complete")
        .json()
        .await
        .expect("activity page response should be typed")
}

async fn post_outcome<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    endpoint: &str,
    request: serde_json::Value,
) -> T {
    client
        .post(endpoint)
        .json(&request)
        .send()
        .await
        .expect("hostile request should receive a typed response")
        .json()
        .await
        .expect("hostile response should decode")
}

fn continuation(outcome: &BankHttpAccountActivityPageOutcome) -> &str {
    match outcome {
        BankHttpAccountActivityPageOutcome::Delivered {
            continuation: Some(token),
            ..
        } => token,
        other => panic!("activity page did not continue: {other:?}"),
    }
}
