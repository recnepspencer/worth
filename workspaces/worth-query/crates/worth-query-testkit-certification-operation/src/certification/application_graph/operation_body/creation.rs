//! Body-authored state and created audit effects share one native candidate.
pub use super::*;
pub use worth_query_host::facade::application_entry::WorthQueryApplicationProgramMutationPreparation as Preparation;
pub use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

pub fn read_audit<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
) -> (u64, u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(audit_query::AuditRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    let row = &result.rows()[0];
    assert_eq!(row.identity, DOCUMENT_IDENTITY);
    (row.retention_days, row.prior_retention_days)
}

pub fn move_audit_source<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    key: u64,
) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let outcome = app
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::SourceIntent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 99,
            ..Default::default()
        }))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }));
}
