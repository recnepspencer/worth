use std::sync::Arc;

use bank_domain::model::{AccountId, InstitutionId};

use super::super::super::protocol::{
    BankHttpAccountActivityItem, BankHttpAccountActivityPageOutcome, BankHttpAccountSummaryOutcome,
    BankHttpCommitDescription, BankHttpCommitDisposition, BankHttpDenialKind,
    BankHttpMutationOutcome, BankHttpMutationRequest, BankHttpPostingPurpose,
};
use super::fixture::application;
use super::{bind_application, controls_json, credential_json, BankHttpServerConfiguration};

#[tokio::test]
async fn malformed_json_returns_a_typed_denial() {
    let server = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let outcome = post_mutation(
        &reqwest::Client::new(),
        &format!("http://{}/v1/mutations", server.local_address()),
        &serde_json::json!({ "protocol": "v1", "request_id": 42 }),
    )
    .await;
    assert!(matches!(
        outcome,
        BankHttpMutationOutcome::NotApplied { denial, .. }
            if denial.kind == BankHttpDenialKind::MalformedRequest
    ));
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn response_loss_reuses_domain_idempotency_without_duplicate_effect() {
    let account = AccountId::new(100).unwrap();
    let institution = InstitutionId::new(1).unwrap();
    let server = bind_application(
        Arc::new(application(account)),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let endpoint = format!("http://{}/v1/mutations", server.local_address());
    let request = deposit_request(institution, account);
    serde_json::from_value::<BankHttpMutationRequest>(request.clone())
        .expect("mutation specimen must match the wire contract");
    let before =
        account_activity(&client, server.local_address(), account, "activity-before").await;
    let lost_response = client
        .post(&endpoint)
        .json(&request)
        .send()
        .await
        .expect("the first deposit should reach the HTTP server");
    assert!(lost_response.status().is_success());
    drop(lost_response);
    let after = account_activity(&client, server.local_address(), account, "activity-after").await;
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!(
        after
            .iter()
            .filter(|entry| {
                entry.purpose == BankHttpPostingPurpose::Deposit && entry.amount_minor == 10
            })
            .count(),
        1,
        "the first publication must expose the deposit exactly once"
    );
    let replayed_wire = client
        .post(&endpoint)
        .json(&request)
        .send()
        .await
        .expect("the idempotent retry should reach the HTTP server")
        .json::<serde_json::Value>()
        .await
        .expect("the retry response should be valid JSON");
    let commit_wire = &replayed_wire["commit"];
    assert!(commit_wire.get("provider_work_units").is_some());
    assert!(commit_wire["provider_work_units"].is_null());
    assert!(commit_wire["invariant_work_units"].as_u64().is_some());
    let replayed: BankHttpMutationOutcome = serde_json::from_value(replayed_wire)
        .expect("the v1 retry response should match the public mutation contract");
    let replayed_description =
        commit_description(&replayed, BankHttpCommitDisposition::AlreadyCommitted);
    assert_eq!(replayed_description.provider_work_units, None);
    assert!(replayed_description.invariant_work_units.is_some());
    assert_eq!(
        account_activity(
            &client,
            server.local_address(),
            account,
            "activity-after-retry"
        )
        .await,
        after,
        "retrying a lost response must not publish another activity entry"
    );
    assert_summary_balance(&client, server.local_address(), account, 310).await;
    server.shutdown().await.expect("server should shut down");
}

fn deposit_request(institution: InstitutionId, account: AccountId) -> serde_json::Value {
    serde_json::json!({
        "protocol": "v1",
        "request_id": "deposit-response-loss",
        "credential": credential_json(),
        "controls": { "deadline_milliseconds": 5_000 },
        "idempotency_key": "deposit-response-loss-key",
        "operation": "deposit",
        "institution": institution.canonical_text(),
        "account": account.canonical_text(),
        "amount_minor_units": 10
    })
}

fn commit_description(
    outcome: &BankHttpMutationOutcome,
    expected: BankHttpCommitDisposition,
) -> BankHttpCommitDescription {
    let BankHttpMutationOutcome::Applied {
        disposition,
        commit,
        ..
    } = outcome
    else {
        panic!("unexpected mutation outcome: {outcome:?}");
    };
    assert_eq!(*disposition, expected);
    *commit
}

async fn account_activity(
    client: &reqwest::Client,
    address: std::net::SocketAddr,
    account: AccountId,
    request_id: &str,
) -> Vec<BankHttpAccountActivityItem> {
    let outcome = client
        .post(format!("http://{address}/v1/queries/account-activity/page"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": request_id,
            "credential": credential_json(),
            "controls": controls_json(16),
            "account": account.canonical_text()
        }))
        .send()
        .await
        .expect("activity request should reach the HTTP server")
        .json::<BankHttpAccountActivityPageOutcome>()
        .await
        .expect("activity response should be typed");
    match outcome {
        BankHttpAccountActivityPageOutcome::Delivered {
            activity,
            continuation: None,
            ..
        } => activity.entries,
        other => panic!("activity page should be complete: {other:?}"),
    }
}

async fn assert_summary_balance(
    client: &reqwest::Client,
    address: std::net::SocketAddr,
    account: AccountId,
    expected: i64,
) {
    let summary = client
        .post(format!("http://{address}/v1/queries/account-summary"))
        .json(&serde_json::json!({
            "protocol": "v1",
            "request_id": "summary-after-deposit",
            "credential": credential_json(),
            "controls": controls_json(1),
            "account": account.canonical_text()
        }))
        .send()
        .await
        .unwrap()
        .json::<BankHttpAccountSummaryOutcome>()
        .await
        .unwrap();
    assert!(matches!(
        summary,
        BankHttpAccountSummaryOutcome::Delivered { summary, .. }
            if summary.current_balance_minor == expected
    ));
}

async fn post_mutation(
    client: &reqwest::Client,
    endpoint: &str,
    request: &serde_json::Value,
) -> BankHttpMutationOutcome {
    client
        .post(endpoint)
        .json(request)
        .send()
        .await
        .expect("mutation request should complete")
        .json()
        .await
        .expect("mutation response should be typed")
}
