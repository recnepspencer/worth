//! Actual native custody survives HTTP reply/listener loss and fresh commands.
use super::super::super::protocol::{
    BankHttpDenialKind, BankHttpMutationOutcome as Outcome, BankHttpMutationRequest,
};
use super::fixture::application;
use super::{bind_application, credential_json, BankHttpServerConfiguration};
use bank_domain::model::AccountId;
use std::sync::Arc;

pub(super) fn create(id: &str, amount: i64) -> serde_json::Value {
    serde_json::json!({ "protocol": "v4", "request_id": id, "credential": credential_json(),
        "command": "create", "custody_id": id, "sequence": 1,
        "controls": { "deadline_milliseconds": 5000, "maximum_history_work": 128 },
        "idempotency_key": format!("key-{id}"),
        "operation": { "operation": "deposit", "institution": "fixture:1", "account": "fixture:100", "amount_minor_units": amount } })
}
pub(super) fn command(id: &str, kind: &str, sequence: u64) -> serde_json::Value {
    let controls = if kind == "resume" {
        serde_json::json!({ "deadline_milliseconds": 5000, "maximum_history_work": 128 })
    } else if kind == "read" {
        serde_json::json!({ "deadline_milliseconds": 5000, "maximum_results": 16, "maximum_work": 20000 })
    } else {
        serde_json::json!({ "deadline_milliseconds": 5000 })
    };
    serde_json::json!({ "protocol": "v4", "request_id": format!("{id}-{sequence}"), "credential": credential_json(),
        "command": kind, "custody_id": id, "sequence": sequence, "controls": controls })
}
pub(super) fn decode(value: serde_json::Value) -> BankHttpMutationRequest {
    serde_json::from_value(value).unwrap()
}
async fn post(client: &reqwest::Client, endpoint: &str, value: &serde_json::Value) -> Outcome {
    client
        .post(endpoint)
        .json(value)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
#[tokio::test]
async fn owning_deposit_partial_lost_reply_fresh_recovery_history_and_close() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let (server, continuation) = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    let endpoint = format!("http://{}/v1/mutations", server.local_address());
    let client = reqwest::Client::new();
    application.runtime.fail_next_durable_append_for_test();
    let lost = client
        .post(&endpoint)
        .json(&create("partial17", 17))
        .send()
        .await
        .unwrap();
    assert_eq!(lost.status(), reqwest::StatusCode::ACCEPTED);
    drop(lost);
    // Actual native close refuses unsettled publication and returns the same owning record.
    assert!(
        matches!(post(&client, &endpoint, &command("partial17", "close", 2)).await,
        Outcome::Refused { denial, accepted_sequence: Some(2), .. } if denial.kind == BankHttpDenialKind::Unavailable)
    );
    let failure = server
        .shutdown()
        .await
        .err()
        .expect("native custody refuses shutdown");
    let server = failure.into_server().unwrap();
    let mut wrong = command("partial17", "resume", 3);
    wrong["credential"]["access_token"] = serde_json::json!("beneficiary");
    assert!(
        matches!(post(&client, &endpoint, &wrong).await, Outcome::Refused { denial, .. } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    // Dropping listener and its error wrapper cannot remove this independent resume entrance.
    drop(server);
    assert!(
        matches!(continuation.execute(decode(command("partial17", "resume", 3))).await, Outcome::Pending { posture, .. } if posture == super::super::super::protocol::BankHttpMutationPosture::RecoveryPerformed)
    );
    assert!(
        matches!(continuation.execute(decode(command("partial17", "resume", 4))).await, Outcome::Pending { posture, .. } if posture == super::super::super::protocol::BankHttpMutationPosture::RetainedDeliveryPending)
    );
    assert!(
        matches!(continuation.execute(decode(command("partial17", "resume", 5))).await, Outcome::Completed { recovered: true, commit, .. } if commit.changed_record_count > 0)
    );
    // A genuinely later publication changes current truth; the retained read stays at deposit17.
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
    let principal = application.authenticate(&scope).await;
    let completed = application
        .runtime
        .mutate(bank_server::mutations::deposit(
            bank_domain::schema::Deposit {
                institution: bank_domain::model::InstitutionId::new(1).unwrap(),
                account: AccountId::new(100).unwrap(),
                amount: bank_domain::model::Money::from_minor(19).unwrap(),
            },
        ))
        .as_principal(&principal)
        .controls(bank_server::BankMutationControls::new(
            scope.clone(),
            bank_domain::proposals::BankIdempotencyKey::new("later19").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|denial| panic!("later declaration: {denial:?}"));
    let request = application.runtime.request(&principal, &scope);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        completed,
    ) = completed.advance(&request, Some(std::num::NonZeroUsize::new(128).unwrap()))
    else {
        panic!("later publication")
    };
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("later cleanup: {denial:?}"));
    let read = continuation
        .execute(decode(command("partial17", "read", 6)))
        .await;
    let Outcome::Delivered { activity, .. } = read else {
        panic!("original retained read: {read:?}")
    };
    assert_eq!(
        activity
            .entries
            .iter()
            .filter(|entry| entry.amount_minor == 17)
            .count(),
        1
    );
    assert_eq!(
        activity.entries.len(),
        3,
        "two fixture publications plus exactly one deposit"
    );
    let mut denied_close = command("partial17", "close", 7);
    denied_close["credential"]["access_token"] = serde_json::json!("beneficiary");
    assert!(
        matches!(continuation.execute(decode(denied_close)).await, Outcome::Refused { denial, accepted_sequence: None, .. } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    assert!(matches!(
        continuation
            .execute(decode(command("partial17", "close", 7)))
            .await,
        Outcome::Closed { .. }
    ));
    assert!(continuation.finish().await.is_ok());
}
#[tokio::test]
async fn owning_money_family_uses_current_declarations_and_lawful_close() {
    let (server, continuation) = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    for (id, operation) in [
        (
            "deposit",
            serde_json::json!({ "operation":"deposit", "institution":"fixture:1", "account":"fixture:100", "amount_minor_units":17 }),
        ),
        (
            "withdraw",
            serde_json::json!({ "operation":"withdraw", "institution":"fixture:1", "account":"fixture:100", "amount_minor_units":17 }),
        ),
        (
            "send",
            serde_json::json!({ "operation":"send_money", "from":"fixture:100", "recipient":"fixture:3", "amount_minor_units":17, "expected_accounting_revision":4, "expected_status":"open" }),
        ),
    ] {
        let mut value = create(id, 17);
        value["operation"] = operation;
        let Outcome::Completed {
            recovered: false,
            commit,
            ..
        } = continuation.execute(decode(value)).await
        else {
            panic!("real money completion: {id}")
        };
        assert_eq!(commit.expected_version_count, usize::from(id == "send"));
        assert_eq!(commit.expected_fact_count, usize::from(id == "send"));
        assert!(matches!(
            continuation.execute(decode(command(id, "close", 2))).await,
            Outcome::Closed { .. }
        ));
    }
    server.shutdown().await.unwrap();
}
#[tokio::test]
async fn owning_transport_capacity_duplicates_and_strict_wire_refuse_before_effect() {
    let (server, continuation) = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral()
            .with_opaque_handle_capacity(std::num::NonZeroUsize::new(1).unwrap()),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    assert!(matches!(
        continuation.execute(decode(create("first", 17))).await,
        Outcome::Completed { .. }
    ));
    assert!(
        matches!(continuation.execute(decode(create("first",99))).await, Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Stale)
    );
    assert!(
        matches!(continuation.execute(decode(create("second",99))).await, Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Saturated)
    );
    let read = continuation
        .execute(decode(command("first", "read", 2)))
        .await;
    assert!(matches!(read, Outcome::Delivered { activity, .. } if activity.entries.len() == 3));
    assert!(
        matches!(continuation.execute(decode(command("first","read",2))).await, Outcome::Refused { denial, .. } if denial.kind == BankHttpDenialKind::Stale)
    );
    let mut old = create("old", 17);
    old["protocol"] = serde_json::json!("v3");
    assert!(
        matches!(continuation.execute(decode(old)).await, Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::UnsupportedProtocol)
    );
    let mut substituted = command("first", "resume", 3);
    substituted["idempotency_key"] = serde_json::json!("other");
    assert!(serde_json::from_value::<BankHttpMutationRequest>(substituted).is_err());
    assert!(matches!(
        continuation
            .execute(decode(command("first", "close", 3)))
            .await,
        Outcome::Closed { .. }
    ));
    server.shutdown().await.unwrap();
}
