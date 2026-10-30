//! Identity/currentness proof, not a second signing fixture or an NCR parity claim.
pub use super::{
    current_snapshot_query,
    operator_identity::{authenticate_operator, request_scope},
    reuse_actions::*,
    revision_query, row_query,
    world::{install, App},
};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{WorthQueryApplicationUncommitted, WorthQueryOperationBodyDenial},
    runtime::ExecutionAllocationPolicy as Allocation,
};
pub fn select(app: &App, input: Select, key: u64) -> Outcome<WorthQueryOperationBodyDenial, ()> {
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
pub fn identity(app: &App) -> impl Eq + std::fmt::Debug + use<> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(current_snapshot_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .1
}
pub fn values(app: &App) -> (u64, u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let row = app
        .request(&principal, &scope)
        .query(row_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0;
    let revision = app
        .request(&principal, &scope)
        .query(revision_query::Read("row-a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0;
    (row, revision)
}
pub fn input(note: Option<&str>, row_value: u64) -> Select {
    Select {
        row: "row-a".into(),
        identity: "same-source".into(),
        value: 10,
        note: note.map(str::to_owned),
        label: "display".into(),
        row_value,
    }
}
