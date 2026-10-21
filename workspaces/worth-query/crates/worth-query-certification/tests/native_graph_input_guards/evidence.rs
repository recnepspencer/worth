//! Named guard defects cross the installed native action and independent query boundaries.
use super::{
    actions::*,
    audit_query,
    operator_identity::{authenticate_operator, request_scope},
    row_query,
    world::{install, App},
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
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
        WorthQueryEntityResolutionRefusalKind as Refusal, WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy,
};
fn input(row: &str, company: &str, value: u64) -> SaveRow {
    SaveRow {
        row: row.into(),
        company: company.into(),
        value,
        calls: None,
    }
}
fn save(
    app: &App,
    key: u64,
    input: SaveRow,
) -> Result<Outcome<WorthQueryOperationBodyDenial, ()>, RequestDenial> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
}
fn read_row(app: &App, row: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(row_query::Read(row.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
pub(super) fn audit(app: &App, row: &str) -> Option<u64> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    match app
        .request(&principal, &scope)
        .query(audit_query::Read(row.into()))
        .execute()
    {
        Ok(result) => Some(result.rows()[0].0),
        Err(QueryDenial::ScopeResolution(cause)) => {
            assert_eq!(cause.kind(), Resolution::Refused(Refusal::UnknownEntity));
            None
        }
        Err(other) => panic!("audit read: {other:?}"),
    }
}
fn change(app: &App, parent: &str, company: &str, key: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .advanced()
        .mutate(ChangeParent {
            parent: parent.into(),
            company: company.into(),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
}
#[test]
fn original_capture_guards_parent_and_identity_without_shifting_write_or_audit_slots() {
    let app = install();
    let calls = Arc::new(AtomicU64::new(0));
    let original = match save(
        &app,
        1,
        SaveRow {
            calls: Some(calls.clone()),
            ..input("row-a", "company-a", 42)
        },
    )
    .unwrap()
    {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("matching parent must commit: {other:?}"),
    };
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        (read_row(&app, "row-a"), audit(&app, "row-a")),
        (42, Some(42))
    );
    change(&app, "parent-a", "changed-company", 2);
    match save(&app, 1, input("row-a", "company-a", 42)).unwrap() {
        Outcome::AlreadyCommitted(replay) => {
            assert_eq!(replay.outcome_identity(), original.outcome_identity())
        }
        other => panic!("original replay must precede today's guard: {other:?}"),
    }
    assert!(
        matches!(save(&app, 3, input("row-a", "company-a", 43)).unwrap(),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch { field, .. }) if field == "Company")
    );
    assert_eq!(
        (read_row(&app, "row-a"), audit(&app, "row-a")),
        (42, Some(42))
    );
}
#[test]
fn missing_ambiguous_absent_and_different_parents_never_publish_row_or_audit() {
    for row in ["row-missing", "row-ambiguous", "row-absent", "row-mismatch"] {
        let app = install();
        match save(&app, 10, input(row, "company-a", 42)) {
            Ok(Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch {
                field,
                ..
            })) if matches!(row, "row-absent" | "row-mismatch") => assert_eq!(field, "Company"),
            Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause)))
                if matches!(row, "row-missing" | "row-ambiguous") =>
            {
                assert_eq!(
                    cause.downcast::<Attempt>().unwrap().kind(),
                    if row == "row-missing" {
                        Kind::MissingAuthoritativeFact
                    } else {
                        Kind::AmbiguousRelation
                    }
                )
            }
            other => panic!("{row}: {other:?}"),
        }
        assert_eq!(read_row(&app, row), 7);
        assert_eq!(audit(&app, row), None);
    }
}
#[test]
fn only_the_selected_parent_field_or_owner_adjacency_can_stale_the_guarded_candidate() {
    for race in ["parent-field", "owner-edge", "unrelated-parent"] {
        let app = install();
        let scope = request_scope();
        let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
        let key = 20;
        let mut request = app
            .request(&principal, &scope)
            .advanced()
            .mutate(input("row-a", "company-a", 42))
            .without_source()
            .idempotency(&key);
        let candidate = match request
            .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
        {
            Preparation::Prepared(candidate) => candidate,
            Preparation::Settled(_) => panic!("fresh guarded candidate"),
        };
        if race == "owner-edge" {
            let result = app
                .request(&principal, &scope)
                .advanced()
                .mutate(IntroduceParent {
                    row: "row-a".into(),
                    identity: "new-parent".into(),
                    optional_company: Some("company-a".into()),
                })
                .without_source()
                .idempotency(&21)
                .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
        } else {
            change(
                &app,
                if race == "parent-field" {
                    "parent-a"
                } else {
                    "parent-b"
                },
                "company-c",
                21,
            );
        }
        let result = candidate.commit(ExecutionAllocationPolicy::SystemAllocation);
        if race == "unrelated-parent" {
            assert!(matches!(result, Outcome::Committed { .. }), "{result:?}");
            assert_eq!(
                (read_row(&app, "row-a"), audit(&app, "row-a")),
                (42, Some(42))
            );
        } else {
            assert!(
                matches!(
                    result,
                    Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
                ),
                "{result:?}"
            );
            assert_eq!((read_row(&app, "row-a"), audit(&app, "row-a")), (7, None));
        }
    }
}
