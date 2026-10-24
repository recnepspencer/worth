//! Native mutation custody serves current money declarations and refuses before effect.
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
        serde_json::json!({ "deadline_milliseconds": 5000, "maximum_results": 16})
    } else {
        serde_json::json!({ "deadline_milliseconds": 5000 })
    };
    serde_json::json!({ "protocol": "v4", "request_id": format!("{id}-{sequence}"), "credential": credential_json(),
        "command": kind, "custody_id": id, "sequence": sequence, "controls": controls })
}
pub(super) fn decode(value: serde_json::Value) -> BankHttpMutationRequest {
    serde_json::from_value(value).unwrap()
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
