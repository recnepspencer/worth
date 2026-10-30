//! A genuine second authentication event cannot replace the natively admitted approver.
pub use super::super::super::document_retention_model::{
    operator_identity::{authenticate_reviewer, block_on, secondary_signer_identity},
    workflow::{WorkflowAdvanceInput, WorkflowApprovalIntent},
};
pub use super::*;
pub use std::time::{Duration, SystemTime};
pub use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial;
pub use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_entry::WorthQueryWorkflowAdvancePreparationDenial,
};

pub struct SecondarySigner;
impl WorthQueryAuthenticationAdapter for SecondarySigner {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "certification-secondary-signer.v1"
    }
    fn validate<'a>(
        &'a self,
        (): (),
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                secondary_signer_identity(),
                WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
                WorthQueryAuthenticationMethod::new("certification").unwrap(),
                now,
                now + Duration::from_secs(3600),
                vec![],
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}
