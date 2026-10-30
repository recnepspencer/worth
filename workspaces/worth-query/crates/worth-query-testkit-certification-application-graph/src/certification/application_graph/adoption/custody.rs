//! Owner-derived custody obligations and the owner gates that enforce them.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
pub use worth_query_host::facade::declaration::application_program::ApplicationSemanticChangeKind;
pub use worth_query_host::facade::domain::WorthQueryProgramCustodyInventoryKind;
pub use worth_query_host::facade::primary_graph::WorthQueryProgramCustodyDispositionKind;

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::publish_on_first_program;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::presented_request::set_retention;
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::{
    ChangedOperationRetentionProgram, RemovedOperationRetentionProgram,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};

/// The binding of a length-framed action subject
/// (`<instance>`, `<feature>`, `<binding>`), compared whole.
pub fn action_binding(subject: &str) -> Option<&str> {
    let mut parts = Vec::new();
    let mut rest = subject;
    while !rest.is_empty() {
        let (length, framed) = rest.split_once(':')?;
        let length = length.parse::<usize>().ok()?;
        parts.push(framed.get(..length)?);
        rest = framed.get(length..)?;
    }
    match parts.as_slice() {
        [_, _, binding] => Some(binding),
        _ => None,
    }
}
