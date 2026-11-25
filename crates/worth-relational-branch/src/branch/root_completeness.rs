//! Check authoritative reconstructed regions against the committed root.
use super::{
    RelationalBranchRoot, RelationalBranchRootAxes, RelationalBranchRootDescriptor,
    RelationalBranchVisibilityCommitment, RelationalRootCommitEvidence,
};
use crate::history::data::CanonicalCommitEnvelope;
use worth_foundational::CanonicalDigestId;

#[derive(Clone, Copy)]
struct ReconstructedContent {
    storage: CanonicalDigestId,
    schema: CanonicalDigestId,
    accumulator: [u8; 32],
}

impl RelationalBranchRoot {
    pub fn is_complete(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> bool {
        let Some(committed) = self.committed.as_ref() else {
            return false;
        };
        let Ok((storage, accumulator)) = self.verify_resident_storage_content(execution, symbols)
        else {
            return false;
        };
        let content = ReconstructedContent {
            storage: CanonicalDigestId::new(storage),
            schema: CanonicalDigestId::new(self.schema_authority.authority_digest()),
            accumulator,
        };
        let schema_matches = committed.evidence.matches_schema(&self.schema_authority);
        match &committed.evidence {
            RelationalRootCommitEvidence::Original(original) => original_is_complete(
                committed.axes,
                &committed.descriptor,
                self.content_accumulator,
                content,
                original.envelope(),
                schema_matches,
            ),
            RelationalRootCommitEvidence::Checkpoint(commit) => bindings_match(
                committed.axes,
                &committed.descriptor,
                (self.content_accumulator, content.accumulator),
                RelationalBranchRootAxes {
                    storage_version: committed.evidence.receipt().version_id.0,
                    storage_root: *content.storage.bytes(),
                    schema_root: *content.schema.bytes(),
                    correctness_index: committed.axes.correctness_index,
                    visibility: commit.visibility(),
                },
                schema_matches,
            ),
        }
    }
}

fn original_is_complete(
    axes: RelationalBranchRootAxes,
    descriptor: &RelationalBranchRootDescriptor,
    accumulator: [u8; 32],
    content: ReconstructedContent,
    body: &CanonicalCommitEnvelope,
    schema_matches: bool,
) -> bool {
    // The production original-evidence predicate recomputes visibility here.
    // Its actual sealed semantic image is observed; no runtime admission is minted.
    let reconstructed = RelationalBranchRootAxes {
        storage_version: body.commit.version_id.0,
        storage_root: *content.storage.bytes(),
        schema_root: *content.schema.bytes(),
        correctness_index: axes.correctness_index,
        visibility: RelationalBranchVisibilityCommitment::for_root(
            body,
            *content.storage.bytes(),
            *content.schema.bytes(),
            axes.correctness_index,
        ),
    };
    bindings_match(
        axes,
        descriptor,
        (accumulator, content.accumulator),
        reconstructed,
        schema_matches,
    )
}

fn bindings_match(
    axes: RelationalBranchRootAxes,
    descriptor: &RelationalBranchRootDescriptor,
    accumulators: ([u8; 32], [u8; 32]),
    reconstructed: RelationalBranchRootAxes,
    schema_matches: bool,
) -> bool {
    axes.storage_root == reconstructed.storage_root
        && accumulators.0 == accumulators.1
        && axes.storage_root == *descriptor.truth_root()
        && axes.schema_root == *descriptor.schema_root()
        && axes.schema_root == reconstructed.schema_root
        && schema_matches
        && axes.storage_version == reconstructed.storage_version
        && axes.storage_root != [0; 32]
        && axes.visibility == reconstructed.visibility
}

#[cfg(test)]
#[path = "root_completeness/tests.rs"]
mod tests;
