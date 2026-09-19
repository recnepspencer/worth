use std::sync::Arc;

use bank_domain::model::AccountId;

use super::super::super::protocol::{
    BankHttpCommitDisposition, BankHttpDenial, BankHttpDenialKind, BankHttpElevationRequestOutcome,
    BankHttpNextAction,
};
use super::super::{bind_application, BankHttpServerConfiguration};
use super::fixture::application;

#[tokio::test]
async fn elevation_request_replays_from_query_after_http_registry_restarts() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let first_server = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap();
    let client = reqwest::Client::new();
    let first_origin = format!("http://{}", first_server.local_address());
    let first = post_request(&client, &first_origin, "first", "prevent_immediate_loss").await;
    let (first_changes, first_emissions) = match first {
        BankHttpElevationRequestOutcome::Requested {
            disposition: BankHttpCommitDisposition::Committed,
            changed_record_count,
            emitted_effect_count,
            ..
        } => (changed_record_count, emitted_effect_count),
        other => panic!("first HTTP request must publish: {other:?}"),
    };
    first_server.shutdown().await.unwrap();

    let resumed_server =
        bind_application(application, BankHttpServerConfiguration::local_ephemeral())
            .await
            .unwrap();
    let resumed_origin = format!("http://{}", resumed_server.local_address());
    let query_drift = post_request(
        &client,
        &resumed_origin,
        "query-drift",
        "meet_legal_deadline",
    )
    .await;
    let query_denial = match query_drift {
        BankHttpElevationRequestOutcome::Denied { denial, .. } => denial,
        other => panic!("changed input must reach Query drift after registry restart: {other:?}"),
    };
    assert_eq!(
        query_denial,
        BankHttpDenial::new(
            BankHttpDenialKind::Stale,
            BankHttpNextAction::CorrectRequest
        )
    );

    let replayed = post_request(&client, &resumed_origin, "replay", "prevent_immediate_loss").await;
    assert!(matches!(
        replayed,
        BankHttpElevationRequestOutcome::Requested {
            disposition: BankHttpCommitDisposition::AlreadyCommitted,
            changed_record_count,
            emitted_effect_count,
            ..
        } if (changed_record_count, emitted_effect_count) == (first_changes, first_emissions)
    ));
    let registry_drift = post_request(
        &client,
        &resumed_origin,
        "registry-drift",
        "meet_legal_deadline",
    )
    .await;
    assert!(matches!(
        registry_drift,
        BankHttpElevationRequestOutcome::Denied { denial, .. } if denial == query_denial
    ));
    resumed_server.shutdown().await.unwrap();
}

async fn post_request(
    client: &reqwest::Client,
    origin: &str,
    request_id: &str,
    reason: &str,
) -> BankHttpElevationRequestOutcome {
    client
        .post(format!("{origin}/v1/estate/elevation/request"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": request_id,
            "credential": {
                "id_token": "test-only",
                "access_token": "test-only",
                "nonce": "test-only"
            },
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "elevation-registry-restart-key",
            "estate": "fixture:3",
            "access": 531,
            "mandatory_review": 532,
            "upper_bound_grant": 20,
            "reason": reason,
            "field": "account_details",
            "duration_seconds": 300
        }))
        .send()
        .await
        .expect("HTTP request should cross TCP")
        .json()
        .await
        .expect("HTTP outcome should decode")
}
