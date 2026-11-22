//! Preserve the original field-retention cause at the ordinary commit boundary.
use super::{denial_cause::DenialCause, WorthQueryApplicationCommitDenial};
use crate::domain_computation::application_aftermath::WorthQueryPreImageRetentionDenial;

impl WorthQueryApplicationCommitDenial {
    /// The installed demand refused prior fields before publication. This is
    /// the original retention result, including its total-byte-bound refusal.
    pub fn preimage_retention_denial(&self) -> Option<WorthQueryPreImageRetentionDenial> {
        match self.cause.as_deref()? {
            DenialCause::ProviderSession(failure) => failure.preimage_retention_denial(),
            _ => None,
        }
    }
}
