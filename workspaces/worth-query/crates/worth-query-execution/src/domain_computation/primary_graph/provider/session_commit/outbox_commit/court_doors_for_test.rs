//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCommittedDispatchOutboxBinding {
    #[doc(hidden)]
    pub fn fixture_from_commit_for_test( layout: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout, record: Option<&WorthQueryDispatchOutboxRecord>, committed: &CommitResult) -> Result<Option<Self>, WorthQueryCommittedDispatchOutboxBindingDenial> {
        Self::fixture_from_commit(layout, record, committed)
    }
}
