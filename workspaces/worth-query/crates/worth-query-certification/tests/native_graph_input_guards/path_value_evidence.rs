//! Shared-parent publication is atomic; replay reuses the original performed result.
use super::{
    audit_query,
    operator_identity::{authenticate_operator, request_scope},
    path_value::BumpRow,
    revision_query, row_query,
    world::{install, App},
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
        WorthQueryApplicationRequestQueryDenial as QueryDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenial as Attempt,
        WorthQueryApplicationAttemptDenialKind as Kind, WorthQueryApplicationUncommitted,
        WorthQueryEntityResolutionDenialKind as Resolution,
        WorthQueryEntityResolutionRefusalKind as Refusal,
    },
    runtime::ExecutionAllocationPolicy,
};
fn row(app: &App, name: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(row_query::Read(name.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
fn revision(app: &App, name: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(revision_query::Read(name.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
fn missing_audit(app: &App, name: &str) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    match app
        .request(&principal, &scope)
        .query(audit_query::Read(name.into()))
        .execute()
    {
        Err(QueryDenial::ScopeResolution(cause)) => {
            assert_eq!(cause.kind(), Resolution::Refused(Refusal::UnknownEntity))
        }
        Err(other) => panic!("audit read failed for wrong reason: {other:?}"),
        Ok(_) => panic!("audit must be absent"),
    }
}
#[test]
fn competing_rows_publish_parent_row_and_audit_once_and_replay_does_not_increment() {
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let source_a = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute_with_expression_context()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let source_b = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-sibling".into()))
        .execute_with_expression_context()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let input_a = BumpRow {
        row: "row-a".into(),
        value: 42,
    };
    let input_b = BumpRow {
        row: "row-sibling".into(),
        value: 43,
    };
    let mut a = app
        .request(&principal, &scope)
        .advanced()
        .mutate(input_a)
        .expect_source(source_a)
        .idempotency(&100);
    let a = match a
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("first row unexpectedly settled"),
    };
    let mut b = app
        .request(&principal, &scope)
        .advanced()
        .mutate(input_b.clone())
        .expect_source(source_b.clone())
        .idempotency(&101);
    let b = match b
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("second row unexpectedly settled"),
    };
    let receipt = match b.commit(ExecutionAllocationPolicy::SystemAllocation) {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("winning row must commit: {other:?}"),
    };
    let loser = a.commit(ExecutionAllocationPolicy::SystemAllocation);
    assert!(
        matches!(
            loser,
            Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
        ),
        "{loser:?}"
    );
    assert_eq!(
        (
            row(&app, "row-a"),
            row(&app, "row-sibling"),
            revision(&app, "row-a"),
            revision(&app, "row-mismatch")
        ),
        (7, 43, 2, 1)
    );
    missing_audit(&app, "row-a");
    assert_eq!(
        app.request(&principal, &scope)
            .query(audit_query::Read("row-sibling".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        2
    );
    let replay = app
        .request(&principal, &scope)
        .advanced()
        .mutate(input_b)
        .expect_source(source_b)
        .idempotency(&101)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    match replay {
        Outcome::AlreadyCommitted(replay) => {
            assert_eq!(replay.outcome_identity(), receipt.outcome_identity())
        }
        other => panic!("original source replay: {other:?}"),
    }
    assert_eq!(revision(&app, "row-a"), 2);
}
#[test]
fn checked_parent_revision_overflow_publishes_neither_row_nor_audit() {
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let source = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-overflow".into()))
        .execute_with_expression_context()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let result = app
        .request(&principal, &scope)
        .advanced()
        .mutate(BumpRow {
            row: "row-overflow".into(),
            value: 42,
        })
        .expect_source(source)
        .idempotency(&102)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation);
    match result {
        Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) => assert_eq!(
            cause.downcast::<Attempt>().unwrap().kind(),
            Kind::OperationBodyValueExpressionDenied
        ),
        other => panic!("checked arithmetic must refuse: {other:?}"),
    }
    assert_eq!(
        (row(&app, "row-overflow"), revision(&app, "row-overflow")),
        (7, u64::MAX)
    );
    missing_audit(&app, "row-overflow");
}

#[test]
fn target_only_assignment_tracks_the_destination_not_just_its_operands() {
    use super::path_value_targets::CopyRow;
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let copy = CopyRow {
        row: "row-a".into(),
    };
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(copy.clone())
        .without_source()
        .idempotency(&103);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh target-only write"),
    };
    let source = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-sibling".into()))
        .execute_with_expression_context()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let landed = app
        .request(&principal, &scope)
        .advanced()
        .mutate(BumpRow {
            row: "row-sibling".into(),
            value: 42,
        })
        .expect_source(source)
        .idempotency(&104)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(landed, Outcome::Committed { .. }), "{landed:?}");
    let stale = candidate.commit(ExecutionAllocationPolicy::SystemAllocation);
    assert!(
        matches!(
            stale,
            Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
        ),
        "{stale:?}"
    );
    assert_eq!((row(&app, "row-a"), revision(&app, "row-a")), (7, 2));
    let committed = app
        .request(&principal, &scope)
        .advanced()
        .mutate(copy)
        .without_source()
        .idempotency(&105)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(committed, Outcome::Committed { .. }),
        "{committed:?}"
    );
    assert_eq!(
        (
            row(&app, "row-a"),
            revision(&app, "row-a"),
            revision(&app, "row-mismatch")
        ),
        (7, 7, 1)
    );
}

#[test]
fn distinct_paths_to_the_same_destination_do_not_become_last_wins() {
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .advanced()
        .mutate(super::path_value_targets::AliasRow {
            row: "row-mismatch".into(),
        })
        .without_source()
        .idempotency(&106)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation);
    match result {
        Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) => assert_eq!(
            cause.downcast::<Attempt>().unwrap().kind(),
            Kind::ConflictingEffectStep
        ),
        other => panic!("aliased authored writes must refuse: {other:?}"),
    }
    assert_eq!(
        (row(&app, "row-mismatch"), revision(&app, "row-mismatch")),
        (7, 1)
    );
}
