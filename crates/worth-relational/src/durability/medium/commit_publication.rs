//! A live commit's distinct memory-only or medium-committed cutover.

use std::sync::Arc;

use worth_proof::DurablyCommitted;

use super::{
    RelationalDurableColdSource, RelationalDurableCommitReadmission,
    RelationalDurableMediumBinding, RelationalDurableStepStop,
};
use crate::history::data::PositionedCanonicalCommit;
use crate::history::RetainedCanonicalIndex;

pub(crate) enum RelationalCommitPublication {
    MemoryOnly {
        positioned: Arc<PositionedCanonicalCommit>,
        original: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact,
    },
    Durable(DurablyCommitted<RelationalDurableCommitReadmission>),
}

impl RelationalCommitPublication {
    pub(crate) fn prepare_memory_frame(
        position: crate::publication::patch::data::PatchStreamPosition,
        prepared: &crate::durability::canonical_commit_artifact::PreparedOriginalCanonicalPayload,
        medium: Option<&RelationalDurableMediumBinding>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact>,
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    > {
        if medium.is_some() {
            return Ok(None);
        }
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_memory_publication(position,prepared,execution).map(Some)
    }
    /// Publish or retain the stream's own subject. A caller cannot substitute
    /// another admitted publication for the prepared route's commit.
    pub(crate) fn publish(
        positioned: Arc<PositionedCanonicalCommit>,
        roots: &crate::branch::RelationalBranchRootDescriptor,
        medium: Option<&RelationalDurableMediumBinding>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: Option<
            crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact,
        >,
    ) -> Result<Self, RelationalDurableStepStop> {
        match medium {
            Some(medium) => medium
                .publish(positioned, roots, policy, execution)
                .map(Self::Durable),
            None => Ok(Self::MemoryOnly {
                positioned,
                original: original.expect(
                    "the performed stream prepared its original Memory frame before admission",
                ),
            }),
        }
    }

    /// Apply the exact admitted subject once. Only the durable branch supplies
    /// journal readmission, and it spends its proof around the live cutover.
    pub(crate) fn cutover<Output>(
        self,
        apply: impl FnOnce(
            Arc<PositionedCanonicalCommit>,
            Option<(RetainedCanonicalIndex, Arc<dyn RelationalDurableColdSource>)>,
            Option<worth_execution::ExecutionImmutableBytes>,
        ) -> Output,
    ) -> Output {
        match self {
            Self::MemoryOnly {
                positioned,
                original,
            } => apply(positioned, None, Some(original.into_bytes())),
            Self::Durable(committed) => committed.consume(|publication| {
                let (positioned, entry, source) = publication.into_parts();
                apply(positioned, Some((entry, source)), None)
            }),
        }
    }
}
