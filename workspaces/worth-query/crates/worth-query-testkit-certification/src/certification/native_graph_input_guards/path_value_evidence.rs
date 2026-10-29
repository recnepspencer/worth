//! Shared-parent publication is atomic; replay reuses the original performed result.
pub use super::{
    audit_query,
    operator_identity::{authenticate_operator, request_scope},
    path_value::BumpRow,
    revision_query, row_query,
    world::{install, App},
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
        WorthQueryEntityResolutionRefusalKind as Refusal,
    },
    runtime::ExecutionAllocationPolicy,
};
pub fn row(app: &App, name: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(row_query::Read(name.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
pub fn revision(app: &App, name: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(revision_query::Read(name.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .0
}
pub fn missing_audit(app: &App, name: &str) {
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
