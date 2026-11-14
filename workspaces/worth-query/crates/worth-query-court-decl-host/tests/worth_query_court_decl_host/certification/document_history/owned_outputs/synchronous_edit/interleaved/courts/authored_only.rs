//! Ordinary actions and real inverse roles need no invented geometry root.
use super::super::super::super::authored_member as member;
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestMutationDenial, WorthQueryPerformedMutationExecutionDenial,
};
mod corrective;

fn cap(value: u64) -> fixture::cap::Intent {
    fixture::cap::Intent {
        key: "envelope-cap".into(),
        value,
        key_derived_target: false,
    }
}

#[test]
fn authored_only_commit_resolves_original_intent_but_cannot_omit_a_required_source() {
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let make = |value, key: &str| {
        let mut edit = request
            .edit_envelope()
            .idempotency(key.to_owned())
            .runtime_stages(&host);
        let token = edit
            .push_stage::<ApplicationNoOutputGraph, _>(cap(value))
            .unwrap();
        (edit, token)
    };
    let before = super::super::super::courts::native_head(&host);
    let mut omitted = request
        .edit_envelope()
        .idempotency("omitted-required-source".to_owned())
        .runtime_stages(&host);
    omitted
        .push_stage::<ApplicationNoOutputGraph, _>(fixture::model::Intent {
            key: "a".into(),
            value: 1001,
            key_derived_target: false,
        })
        .unwrap();
    let (refused, writes) = observe_maintenance_work(8, || {
        omitted.resolve_idempotency(ExecutionAllocationPolicy::SystemAllocation)
    });
    let cause = refused.unwrap_err();
    assert!(
        matches!(
            cause.cause(),
            WorthQueryPerformedMutationExecutionDenial::Mutation(
                WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired
            )
        ),
        "{cause:?}"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(super::super::super::courts::native_head(&host), before);
    let (unseen, _) = make(8, "authored-only");
    assert!(matches!(
        unseen
            .resolve_idempotency(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Unseen
    ));
    let (edit, token) = make(8, "authored-only");
    let (outcome, writes) = observe_maintenance_work(8, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(original) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        (
            original.stage_count(),
            token.ordinal(),
            original.result(&token).unwrap().value
        ),
        (1, 0, 8)
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(
        super::super::super::courts::native_head(&host).1,
        [before.0]
    );
    let (later, token) = make(9, "authored-later");
    let Outcome::Committed(later) = later
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("later ordinary edit refused")
    };
    assert_eq!(later.result(&token).unwrap().value, 9);
    let current = super::super::super::courts::native_head(&host);
    let (_, writes) = observe_maintenance_work(8, || {
        let (retry, _) = make(8, "authored-only");
        let Outcome::RecoveredCommitted(recorded) = retry
            .resolve_idempotency(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
        else {
            panic!("original intent did not resolve")
        };
        assert!(original.receipt().matches_recorded_original(&recorded));
        let (drift, _) = make(1001, "authored-only");
        assert!(matches!(
            drift
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap(),
            Outcome::IdempotencyIntentDrift
        ));
        // The same binding/input with an actual graph is a different whole intent.
        let drift = request
            .edit_envelope()
            .stage::<ModelTree, _>(cap(8))
            .idempotency("authored-only".to_owned())
            .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(drift, Outcome::IdempotencyIntentDrift),
            "{drift:?}"
        );
    });
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(super::super::super::courts::native_head(&host), current);
}
