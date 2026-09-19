use std::sync::Arc;

use bank_domain::model::AccountId;

use super::super::super::protocol::{
    BankHttpAccountActivityPageOutcome, BankHttpAccountSummaryOutcome, BankHttpDenialKind,
    BankHttpEstateNotificationOutcome, BankHttpMutationOutcome, BankHttpQueryBasisPosture,
    BankHttpQueryCapabilityPurpose, BankHttpQueryDisclosurePosture, BankHttpQueryOmissionPosture,
};
use super::super::{bind_application, BankHttpServerConfiguration};
use super::fixture::held_authentication_application;
use super::{application, controls_json, credential_json, post_outcome};

#[tokio::test]
async fn authenticated_account_summary_crosses_the_bounded_tcp_boundary() {
    let account = AccountId::new(100).unwrap();
    let server = bind_application(
        Arc::new(application(account)),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let response = reqwest::Client::new()
        .post(format!(
            "http://{}/v1/queries/account-summary",
            server.local_address()
        ))
        .json(&serde_json::json!({
            "protocol": "v1", "request_id": "account-summary-1",
            "credential": credential_json(),
            "controls": { "deadline_milliseconds": 5_000,
                "maximum_results": 1, "maximum_work": 20_000 },
            "account": account.canonical_text()
        }))
        .send()
        .await
        .expect("TCP request should complete");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    match response
        .json::<BankHttpAccountSummaryOutcome>()
        .await
        .expect("typed response should decode")
    {
        BankHttpAccountSummaryOutcome::Delivered {
            request_id,
            summary,
            publication,
        } => {
            assert_eq!(request_id, "account-summary-1");
            assert_eq!(summary.account, account.canonical_text());
            assert_eq!(summary.display_name, "Daily");
            assert_eq!(summary.current_balance_minor, 300);
            assert_eq!(publication.query_identity.len(), 64);
            assert_eq!(publication.parameter_binding_identity.len(), 64);
            assert_eq!(
                publication.basis.posture,
                BankHttpQueryBasisPosture::SelectedProduct
            );
            assert!(!publication.basis.branch.is_empty());
            assert_eq!(
                publication.capability_purpose,
                BankHttpQueryCapabilityPurpose::AccountServicing
            );
            assert_eq!(
                publication.disclosure.posture,
                BankHttpQueryDisclosurePosture::Public
            );
            assert_eq!(
                publication.disclosure.omission,
                BankHttpQueryOmissionPosture::NoOmission
            );
        }
        BankHttpAccountSummaryOutcome::Denied { denial, .. } => {
            panic!("lawful request was denied: {denial:?}")
        }
    }
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn unknown_protocol_version_returns_the_typed_upgrade_denial() {
    let account = AccountId::new(100).unwrap();
    let server = bind_application(
        Arc::new(application(account)),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let outcome = reqwest::Client::new()
        .post(format!(
            "http://{}/v1/queries/account-summary",
            server.local_address()
        ))
        .json(&serde_json::json!({
            "protocol": "v99", "request_id": "unsupported-version",
            "credential": credential_json(), "controls": controls_json(1),
            "account": account.canonical_text()
        }))
        .send()
        .await
        .expect("unknown version should receive a typed response")
        .json::<BankHttpAccountSummaryOutcome>()
        .await
        .expect("unknown version response should decode");
    assert!(matches!(
        outcome,
        BankHttpAccountSummaryOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::UnsupportedProtocol
    ));
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn full_request_queue_rejects_before_authentication_or_domain_work() {
    let account = AccountId::new(100).unwrap();
    let (application, hold) = held_authentication_application(account);
    let server = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral()
            .with_queue_capacity(std::num::NonZeroUsize::new(1).unwrap())
            .with_maximum_concurrency(std::num::NonZeroUsize::new(1).unwrap()),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let endpoint = format!(
        "http://{}/v1/queries/account-summary",
        server.local_address()
    );
    let first = tokio::spawn(post_summary(
        client.clone(),
        endpoint.clone(),
        account,
        "queue-held-first".to_owned(),
    ));
    hold.wait_for_calls(1).await;

    let attempts = (0..4)
        .map(|ordinal| {
            tokio::spawn(post_summary(
                client.clone(),
                endpoint.clone(),
                account,
                format!("queue-contender-{ordinal}"),
            ))
        })
        .collect::<Vec<_>>();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    hold.release(2);

    let mut outcomes = Vec::new();
    outcomes.push(first.await.expect("held request task should join"));
    for attempt in attempts {
        outcomes.push(attempt.await.expect("contender task should join"));
    }
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(
                outcome,
                BankHttpAccountSummaryOutcome::Denied { denial, .. }
                    if denial.kind == BankHttpDenialKind::Saturated
            ))
            .count(),
        3,
        "one request may execute and one may queue; all later work must fail closed"
    );
    assert_eq!(
        hold.call_count(),
        2,
        "saturated work must not cross authentication or reach the Bank runtime"
    );
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn authority_shaped_unknown_fields_fail_closed_across_wire_families() {
    let account = AccountId::new(100).unwrap();
    let server = bind_application(
        Arc::new(application(account)),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let origin = format!("http://{}", server.local_address());

    let summary: BankHttpAccountSummaryOutcome = post_outcome(
        &client,
        &format!("{origin}/v1/queries/account-summary"),
        serde_json::json!({
            "protocol": "v1", "request_id": "unknown-summary",
            "credential": credential_json(), "controls": controls_json(1),
            "account": account.canonical_text(), "branch": "forged"
        }),
    )
    .await;
    assert!(
        matches!(summary, BankHttpAccountSummaryOutcome::Denied { denial, .. }
        if denial.kind == BankHttpDenialKind::MalformedRequest)
    );

    let page: BankHttpAccountActivityPageOutcome = post_outcome(
        &client,
        &format!("{origin}/v1/queries/account-activity/page"),
        serde_json::json!({
            "protocol": "v1", "request_id": "unknown-page",
            "credential": credential_json(), "controls": controls_json(1),
            "account": account.canonical_text(), "provider": "forged"
        }),
    )
    .await;
    assert!(
        matches!(page, BankHttpAccountActivityPageOutcome::Denied { denial, .. }
        if denial.kind == BankHttpDenialKind::MalformedRequest)
    );

    let notification: BankHttpEstateNotificationOutcome = post_outcome(
        &client,
        &format!("{origin}/v1/estate/notify-death"),
        serde_json::json!({
            "protocol": "v1", "request_id": "unknown-notification",
            "credential": credential_json(),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "unknown-notification-key",
            "estate": "fixture:3", "notice": "fixture:12", "subject": "fixture:2",
            "authority": "forged"
        }),
    )
    .await;
    assert!(
        matches!(notification, BankHttpEstateNotificationOutcome::Denied { denial, .. }
        if denial.kind == BankHttpDenialKind::MalformedRequest)
    );

    let mutation: BankHttpMutationOutcome = post_outcome(
        &client,
        &format!("{origin}/v1/mutations"),
        serde_json::json!({
            "protocol": "v1", "request_id": "unknown-mutation",
            "credential": credential_json(),
            "controls": { "deadline_milliseconds": 5_000 },
            "idempotency_key": "unknown-mutation-key", "operation": "deposit",
            "institution": "fixture:1", "account": account.canonical_text(),
            "amount_minor_units": 1, "branch": "forged"
        }),
    )
    .await;
    assert!(
        matches!(mutation, BankHttpMutationOutcome::NotApplied { denial, .. }
        if denial.kind == BankHttpDenialKind::MalformedRequest)
    );

    server.shutdown().await.expect("server should shut down");
}

async fn post_summary(
    client: reqwest::Client,
    endpoint: String,
    account: AccountId,
    request_id: String,
) -> BankHttpAccountSummaryOutcome {
    client
        .post(endpoint)
        .json(&serde_json::json!({
            "protocol": "v1", "request_id": request_id,
            "credential": credential_json(), "controls": controls_json(1),
            "account": account.canonical_text()
        }))
        .send()
        .await
        .expect("queue request should respond")
        .json()
        .await
        .expect("queue outcome should remain typed")
}
