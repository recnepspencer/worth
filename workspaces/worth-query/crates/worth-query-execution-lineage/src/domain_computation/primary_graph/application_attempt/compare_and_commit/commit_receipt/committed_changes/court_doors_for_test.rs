//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommittedChanges {
    #[doc(hidden)]
    pub fn shares_original_for_test(&self, other: &Self) -> bool {
        Self::shares_original(self, other)
    }
}

impl WorthQueryApplicationCommittedChanges {
    #[doc(hidden)]
    pub fn committed_field_values_for_test(
        &self,
        entity: EntityId,
        fields: &[&AspectFieldLocator],
    ) -> Option<Vec<AspectValue>> {
        Self::committed_field_values(self, entity, fields)
    }
}

impl WorthQueryApplicationCommittedChanges {
    #[doc(hidden)]
    pub fn from_commit_for_test(commit: &CommitResult) -> Self {
        Self::from_commit(commit)
    }
}

impl WorthQueryApplicationCommittedChanges {
    /// Read-only original published rows, for the closed semantic allocation oracle.
    #[doc(hidden)]
    pub fn record_patches_for_test(&self) -> &[PublishedAuthoritativeRecordPatch] {
        &self.envelope.patch.authoritative_record_patches
    }
}
