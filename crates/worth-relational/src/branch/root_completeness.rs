//! Check authoritative reconstructed regions against the committed root.
use super::{
    axes, RelationalBranchRoot, RelationalBranchVisibilityCommitment, RelationalRootCommitEvidence,
};

impl RelationalBranchRoot {
    pub(crate) fn is_complete(&self, symbols: &crate::symbols::data::StringInterner) -> bool {
        let Some(committed) = self.committed.as_ref() else {
            return false;
        };
        let Ok((reconstructed_root, reconstructed_accumulator)) =
            axes::storage_root_from_authoritative_regions(&self.regions, symbols)
        else {
            return false;
        };
        let canonical_schema_root = self.schema_authority.authority_digest();
        let reconstructed_visibility = match &committed.evidence {
            RelationalRootCommitEvidence::Original(envelope) => {
                RelationalBranchVisibilityCommitment::for_root(
                    envelope,
                    reconstructed_root,
                    canonical_schema_root,
                    committed.axes.correctness_index,
                )
            }
            RelationalRootCommitEvidence::Checkpoint(commit) => commit.visibility(),
        };
        committed.axes.storage_root == reconstructed_root
            && self.content_accumulator == reconstructed_accumulator
            && committed.axes.storage_root == *committed.descriptor.truth_root()
            && committed.axes.schema_root == *committed.descriptor.schema_root()
            && committed.axes.schema_root == canonical_schema_root
            && committed.evidence.matches_schema(&self.schema_authority)
            && committed.axes.storage_version == committed.evidence.receipt().version_id.0
            && committed.axes.storage_root != [0; 32]
            && committed.axes.visibility == reconstructed_visibility
    }
}
