//! Sealed test doors over the provider's completed evidence.
use super::super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn completed_evidence_retained_for_test(&self) -> usize {
        Self::completed_evidence_retained(self)
    }
}
