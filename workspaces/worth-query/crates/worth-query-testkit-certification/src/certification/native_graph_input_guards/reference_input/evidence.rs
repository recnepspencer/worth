//! No-source binding isolates native reference CAS from query-source freshness checks.
pub use super::super::{
    operator_identity::{authenticate_operator, request_scope},
    row_query,
    world::{install, App},
};
pub use super::{query, Assign};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationUncommitted as Uncommitted,
        WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind as Resolution,
        WorthQueryEntityResolutionRefusalKind as Refusal, WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
pub fn input(row: &str, selected: Option<&str>) -> Assign {
    Assign {
        row: row.into(),
        selected: selected.map(str::to_owned),
        encoding: super::declaration::Encoding::Normal,
    }
}
pub fn assign(
    app: &App,
    input: Assign,
    key: u64,
) -> Result<Outcome<WorthQueryOperationBodyDenial, ()>, RequestDenial> {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, Allocation::SystemAllocation)
}
pub fn read(app: &App, row: &str) -> Vec<query::Value> {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(query::Read(row.into()))
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}
