//! Actual incoming-list decisions retain empty membership and model field support.
use super::*;
use model::{adjacency_link::Link, authored_identity_model::Create, other_edit::OtherEdit};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

fn seed_model(host: &host::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let key = "seed-adjacency-model".to_owned();
    let result = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Create {
            key: "numeric".into(),
            name: "adjacency-model".into(),
            payload: 41,
            inverse_target: None,
            collateral: None,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        result,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

fn link(host: &host::Host, present: bool, key: &str) -> WorthQueryApplicationCommitReceipt {
    let key = key.to_owned();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    match host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Link {
            key: "adjacency-model".into(),
            other: "numeric".into(),
            present,
            primary: if present { 42 } else { 41 },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("actual link operation: {other:?}"),
    }
}

fn primary(host: &host::Host) -> u64 {
    // The common read helper selects the unique numeric root by its scope.
    host::read(host)[0].primary
}

#[test]
fn restored_adjacency_reversal_allows_the_original_field_source_to_reverse() {
    let host = host::install();
    seed_model(&host);
    host::edit(&host, Value::Primary(3), "adjacency-initial");
    // Incoming numeric membership is empty: seed creation linked in the
    // opposite direction, numeric -> model. Future entrants must be selected.
    let a = host::edit(&host, Value::AdjacentPrimary(7), "adjacency-a");
    let _a = pin_current(&host, &a);
    let b = link(&host, true, "adjacency-entrant-b");
    let _b = pin_current(&host, &b);
    let reversed = reverse(&host, &b, "reverse-adjacency-b");
    let _reversed = pin_current(&host, &reversed);
    assert_eq!(
        reversed
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        b.commit_reference()
    );
    assert_eq!(primary(&host), 7);
    reverse(&host, &a, "reverse-adjacency-a");
    assert_eq!(primary(&host), 3);
}

#[test]
fn live_membership_round_trip_and_authored_noop_do_not_cancel_equal_support() {
    for noop in [false, true] {
        let host = host::install();
        seed_model(&host);
        host::edit(&host, Value::Primary(3), "live-adjacency-initial");
        let a = host::edit(&host, Value::AdjacentPrimary(7), "live-adjacency-a");
        let _a = pin_current(&host, &a);
        let mut pins = Vec::new();
        if noop {
            let equal = host::edit(&host, Value::AdjacentPrimary(7), "live-adjacency-noop");
            pins.push(pin_current(&host, &equal));
        } else {
            let entrant = link(&host, true, "live-adjacency-entrant");
            pins.push(pin_current(&host, &entrant));
            let removed = link(&host, false, "live-adjacency-removal");
            pins.push(pin_current(&host, &removed));
        }
        assert_eq!(primary(&host), 7);
        diverged(&host, &a);
    }
}

#[test]
fn model_resolution_remains_an_independent_dependency_after_later_reverse() {
    let host = host::install();
    seed_model(&host);
    link(&host, true, "seed-model-membership");
    host::edit(&host, Value::Primary(3), "model-initial");
    let a = host::edit(&host, Value::AdjacentPrimary(7), "model-a");
    let _a = pin_current(&host, &a);
    let b = host::edit(&host, Value::Primary(8), "model-b");
    let _b = pin_current(&host, &b);
    let reversed = reverse(&host, &b, "reverse-model-b");
    let _reversed = pin_current(&host, &reversed);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let resolution_key = "change-model-resolution".to_owned();
    let changed = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(OtherEdit {
            key: "adjacency-model".into(),
            value: Value::Ceiling(50),
        })
        .without_source()
        .idempotency(&resolution_key)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(primary(&host), 7);
    diverged(&host, &a);
}
