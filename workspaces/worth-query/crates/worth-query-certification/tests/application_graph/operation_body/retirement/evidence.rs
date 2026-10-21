//! Each case protects one native source-consequence boundary, not NCR parity.
use super::{super::*, binding::ReplaceIntent, read::SnapshotRead};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
        WorthQueryApplicationRequestQueryDenial as QueryDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenial as Attempt,
        WorthQueryApplicationAttemptDenialKind as Kind, WorthQueryApplicationUncommitted,
    },
};
type App<const M: u64> =
    WorthQueryProgramApplicationRuntime<DocumentRetentionSchema<M>, program::BodyProgram<M>>;

fn input(value: u64) -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days: value,
        ..Default::default()
    }
}
fn replace<const M: u64>(app: &App<M>, holder: &str, value: u64, key: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let outcome = app
        .request(&principal, &scope)
        .advanced()
        .mutate(ReplaceIntent(SetRetentionInput {
            identity: holder.into(),
            ..input(value)
        }))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}
fn snapshot<const M: u64>(app: &App<M>, value: u64, present: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    match app
        .request(&principal, &scope)
        .query(SnapshotRead(value))
        .execute()
    {
        Ok(result) => {
            assert!(present, "retired snapshot {value} survived");
            assert_eq!(result.rows()[0].0, value);
        }
        Err(QueryDenial::ScopeResolution(cause)) if !present => {
            assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity));
        }
        Err(other) => panic!("snapshot {value}, expected present={present}: {other:?}"),
    }
}
fn audit<const M: u64>(app: &App<M>, identity: &str) -> Option<(u64, u64)> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    match app
        .request(&principal, &scope)
        .query(audit_query::AuditRead {
            identity: identity.into(),
        })
        .execute()
    {
        Ok(result) => Some((
            result.rows()[0].retention_days,
            result.rows()[0].prior_retention_days,
        )),
        Err(QueryDenial::ScopeResolution(cause)) => {
            assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity));
            None
        }
        Err(other) => panic!("audit read: {other:?}"),
    }
}
#[test]
fn value_change_retires_all_selections_preserves_holders_and_replays_original_commit() {
    let app = install::<120>();
    replace(&app, "holder-b", 202, 1200);
    assert!(matches!(
        invoke_input(&app, 1201, input(7)),
        Outcome::Committed { .. }
    ));
    snapshot(&app, 101, true);
    snapshot(&app, 202, true);
    let calls = Arc::new(AtomicU64::new(0));
    let original = match invoke_input(
        &app,
        1202,
        SetRetentionInput {
            serialization_calls: Some(calls.clone()),
            ..input(8)
        },
    ) {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("source consequence: {other:?}"),
    };
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(read_retention(&app), 8);
    snapshot(&app, 101, false);
    snapshot(&app, 202, false);
    snapshot(&app, 999, true);
    for holder in ["holder-a", "holder-b", "holder-outside"] {
        assert_eq!(audit(&app, holder), Some((7, 7)));
    }
    // Audit identities are intentionally not unique: this fixture is not a business audit model.
    // The first change's audit is asserted independently below in the fresh-app cases.
    replace(&app, "holder-a", 303, 1203); // An unremoved dangling pointer would refuse replacement.
    snapshot(&app, 303, true);
    assert!(matches!(
        invoke_input(&app, 1204, input(9)),
        Outcome::Committed { .. }
    ));
    snapshot(&app, 303, false);
    match invoke_input(&app, 1202, input(8)) {
        Outcome::AlreadyCommitted(replay) => {
            assert_eq!(original.outcome_identity(), replay.outcome_identity())
        }
        other => panic!("original replay: {other:?}"),
    }
    assert_eq!(read_retention(&app), 9);
}
#[test]
fn inner_selection_movement_stales_source_cleanup_and_audit_together() {
    let app = install::<120>();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 1210;
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::BodyIntent(input(8)))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh change must prepare"),
    };
    replace(&app, "holder-b", 203, 1211); // Empty inner selection was part of the prepared observation.
    assert!(matches!(
        candidate.commit(ExecutionAllocationPolicy::SystemAllocation),
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    ));
    assert_eq!(read_retention(&app), 7);
    snapshot(&app, 101, true);
    snapshot(&app, 203, true);
    snapshot(&app, 999, true);
    assert_eq!(audit(&app, DOCUMENT_IDENTITY), None);
    assert_eq!(audit(&app, "holder-a"), Some((7, 7)));
    assert!(matches!(
        invoke_input(&app, 1212, input(8)),
        Outcome::Committed { .. }
    ));
    assert_eq!(audit(&app, DOCUMENT_IDENTITY), Some((8, 7)));
    snapshot(&app, 101, false);
    snapshot(&app, 203, false);
}
fn refusal<const M: u64>(expected: Kind) {
    let app = install::<M>();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::BodyIntent(input(8)))
        .without_source()
        .idempotency(&1220)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation);
    let Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) = result else {
        panic!("must refuse before publication: {result:?}");
    };
    assert_eq!(cause.downcast::<Attempt>().unwrap().kind(), expected);
    assert_eq!(read_retention(&app), 7);
    snapshot(&app, 101, true);
    snapshot(&app, 999, true);
    assert_eq!(audit(&app, "holder-a"), Some((7, 7)));
    assert_eq!(audit(&app, DOCUMENT_IDENTITY), None);
}
#[test]
fn bound_overflow_and_overlapping_targets_never_apply_a_prefix() {
    refusal::<121>(Kind::CandidateCapacityExceeded);
    refusal::<122>(Kind::ConflictingEffectStep);
}
#[test]
fn empty_collection_and_optional_trigger_use_original_input_slots() {
    let empty = install::<123>();
    assert!(matches!(
        invoke_input(&empty, 1230, input(8)),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&empty), 8);
    assert_eq!(audit(&empty, DOCUMENT_IDENTITY), Some((8, 7)));
    snapshot(&empty, 999, true);
    let optional = install::<124>();
    assert!(matches!(
        invoke_input(&optional, 1240, input(8)),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&optional), 8);
    snapshot(&optional, 101, false);
    snapshot(&optional, 999, true);
    assert_eq!(audit(&optional, DOCUMENT_IDENTITY), Some((8, 7)));
}
