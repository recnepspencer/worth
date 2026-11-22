//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCanonicalInboundCompletion {
    #[doc(hidden)]
    pub fn authenticated_message_identity_for_test(&self) -> Option<&[u8; 32]> {
        Self::authenticated_message_identity(self)
    }
}

impl WorthQueryCanonicalInboundCompletion {
    #[doc(hidden)]
    pub fn completion_world_commit_for_test(&self) -> &CompositeCommitIdentity {
        Self::completion_world_commit(self)
    }
}

impl WorthQueryCanonicalInboundCompletion {
    #[doc(hidden)]
    pub fn matches_original_dispatch_for_test(
        &self,
        record: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        original_relational_commit: &RelationalCommitReceipt,
        original_world_commit: &CompositeCommitIdentity,
        original_incarnation: ProductBranchIncarnation,
    ) -> bool {
        Self::matches_original_dispatch(
            self,
            record,
            original_relational_commit,
            original_world_commit,
            original_incarnation,
        )
    }
}

impl WorthQueryCanonicalInboundCompletion {
    #[doc(hidden)]
    pub fn operation_for_test(&self) -> &str {
        Self::operation(self)
    }
}

impl WorthQueryCanonicalInboundCompletion {
    #[doc(hidden)]
    pub fn original_world_commit_for_test(&self) -> &CompositeCommitIdentity {
        Self::original_world_commit(self)
    }
}
