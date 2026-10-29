//! Atomic topology through installed native actions, never fixture effect assembly.
pub use super::{
    child_actions::CreateTree,
    child_query,
    operator_identity::{authenticate_operator, request_scope},
    revision_query, row_query,
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
pub fn input(row: &str) -> CreateTree {
    CreateTree {
        parent: "parent-a".into(),
        row: row.into(),
        physical: 101,
        electronic: 102,
        physical_note: Some("held".into()),
        electronic_note: None,
        detail: "physical detail".into(),
    }
}
pub fn execute(
    app: &App,
    input: CreateTree,
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
pub fn cells(app: &App, row: &str) -> Vec<child_query::Value> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(child_query::Read(row.into()))
        .execute()
        .unwrap();
    let mut cells = result.rows().to_vec();
    cells.sort_by_key(|cell| cell.kind);
    cells
}
pub fn revision(app: &App) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
pub fn missing_row(app: &App, row: &str) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(row_query::Read(row.into()))
        .execute();
    assert!(
        matches!(result,Err(QueryDenial::ScopeResolution(cause)) if cause.kind()==Resolution::Refused(Refusal::UnknownEntity))
    );
}
