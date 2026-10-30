//! Fresh ordinary request recovery reads require the current program.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRecoveryRequestDenial, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial, WorthQueryBranchAdoptionPublicationOutcome,
};
pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

pub use super::document_retention_model::{
    host::{publish_on_first_program, publish_on_second_program},
    operator_identity::{authenticate_operator, request_scope},
    programs::RemovedOperationRetentionProgram,
    readback::observe_head,
    retention_entry::{ReviewedSetRetentionIntent, SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

pub fn intent(retention_days: u64) -> SetRetentionIntent {
    SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days,
        },
    }
}
