//! Asking one occurrence to set a document's retention.
//!
//! The request never names a program: the same request, value and host reach
//! a different answer depending on which program the occurrence runs.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::operator_identity::{authenticate_operator, request_scope};
use super::retention_entry::{
    DocumentRetentionWritten, SetRetentionDenial, SetRetentionIntent, DOCUMENT_IDENTITY,
};
use super::schema::{DocumentRetentionSchema, SetRetentionInput};

/// What one set-retention request settled as.
pub type RetentionOutcome =
    WorthQueryApplicationMutationOutcome<SetRetentionDenial, DocumentRetentionWritten>;

/// Issues one set-retention request on one occurrence through the installed
/// owner of the program that occurrence runs.
pub fn set_retention<Program>(
    application: &WorthQueryProgramApplicationRuntime<DocumentRetentionSchema, Program>,
    branch: WorthQueryProductBranch,
    retention_days: u64,
    idempotency: u64,
) -> Result<RetentionOutcome, WorthQueryApplicationRequestMutationDenial>
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema>,
{
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
}
