//! Current-selection disposal is native, atomic and replay-safe; retained history stays distinct.
pub use super::{
    current_snapshot_query,
    operator_identity::{authenticate_operator, request_scope},
    retiring_actions::*,
    row_query, snapshot_query,
    world::{install, App},
};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial as QueryDenial,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted, WorthQueryEntityResolutionDenialKind as Resolution,
        WorthQueryEntityResolutionRefusalKind as Refusal, WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
pub fn retire(
    app: &App,
    input: RetireSnapshot,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, Allocation::SystemAllocation)
        .unwrap()
}
pub fn keep(app: &App, row: &str, identity: &str, value: u64, key: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let outcome = app
        .request(&principal, &scope)
        .advanced()
        .mutate(KeepSnapshot {
            row: row.into(),
            identity: identity.into(),
            value,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, Allocation::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
}
pub fn snapshot(app: &App, identity: &str) -> Option<u64> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    match app
        .request(&principal, &scope)
        .query(snapshot_query::Read(identity.into()))
        .execute()
    {
        Ok(view) => Some(view.rows()[0].0),
        Err(QueryDenial::ScopeResolution(cause)) => {
            assert_eq!(cause.kind(), Resolution::Refused(Refusal::UnknownEntity));
            None
        }
        Err(other) => panic!("snapshot read: {other:?}"),
    }
}
pub fn current(app: &App, row: &str) -> String {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(current_snapshot_query::Read(row.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
        .clone()
}
pub fn row_value(app: &App, row: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(row_query::Read(row.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
pub fn input(row: &str, identity: &str, value: u64) -> RetireSnapshot {
    RetireSnapshot {
        row: row.into(),
        identity: identity.into(),
        value,
    }
}
