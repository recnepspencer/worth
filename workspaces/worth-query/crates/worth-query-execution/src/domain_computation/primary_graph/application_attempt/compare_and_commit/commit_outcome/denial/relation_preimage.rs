//! Preserve the source occurrence's relation-material refusal through commit admission.
use super::*;
impl WorthQueryApplicationCommitDenial {
    pub fn relation_preimage_failure(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRelationPreImageFailure>
    {
        match self.cause.as_deref() {
            Some(denial_cause::DenialCause::ProviderSession(failure)) => {
                failure.relation_preimage_failure()
            }
            _ => None,
        }
    }
}
