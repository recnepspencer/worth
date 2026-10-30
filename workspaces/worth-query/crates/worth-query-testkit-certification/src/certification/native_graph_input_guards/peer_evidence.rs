//! Independent audit effects remain atomic and are not swallowed by snapshot reuse.
pub use super::{
    operator_identity::{authenticate_operator, request_scope},
    peer_audit,
    reuse_evidence::{identity, input, select, values},
    world::{install, App},
};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted, WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
pub fn journal(app: &App, parent: &str) -> Vec<peer_audit::Value> {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(peer_audit::Read(parent.into()))
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}
