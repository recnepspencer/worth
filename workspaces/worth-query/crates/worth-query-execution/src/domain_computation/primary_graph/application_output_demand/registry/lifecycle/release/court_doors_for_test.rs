//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn release_prepared_token_for_test(
        &self, source_commit: &worth_runtime_world::facade::CompositeCommitIdentity) {
        Self::release_prepared_token(self, source_commit)
    }
}
