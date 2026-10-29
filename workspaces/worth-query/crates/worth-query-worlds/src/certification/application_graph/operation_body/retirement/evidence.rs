//! Each case protects one native source-consequence boundary, not NCR parity.
pub use super::{super::*, binding::ReplaceIntent, read::SnapshotRead};
pub use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
pub use worth_query_host::facade::{
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
pub type App<const M: u64> =
    WorthQueryProgramApplicationRuntime<DocumentRetentionSchema<M>, program::BodyProgram<M>>;

pub fn input(value: u64) -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days: value,
        ..Default::default()
    }
}
pub fn replace<const M: u64>(app: &App<M>, holder: &str, value: u64, key: u64) {
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
pub fn snapshot<const M: u64>(app: &App<M>, value: u64, present: bool) {
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
pub fn audit<const M: u64>(app: &App<M>, identity: &str) -> Option<(u64, u64)> {
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

pub fn refusal<const M: u64>(expected: Kind) {
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
