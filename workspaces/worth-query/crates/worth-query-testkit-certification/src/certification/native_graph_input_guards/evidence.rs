//! Named guard defects cross the installed native action and independent query boundaries.
pub use super::{
    actions::*,
    audit_query,
    operator_identity::{authenticate_operator, request_scope},
    row_query,
    world::{install, App},
};
pub use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
pub use worth_query_host::facade::{
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
pub fn input(row: &str, company: &str, value: u64) -> SaveRow {
    SaveRow {
        row: row.into(),
        company: company.into(),
        value,
        calls: None,
    }
}
pub fn save(
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
pub fn read_row(app: &App, row: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(row_query::Read(row.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
pub fn audit(app: &App, row: &str) -> Option<u64> {
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
pub fn change(app: &App, parent: &str, company: &str, key: u64) {
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
