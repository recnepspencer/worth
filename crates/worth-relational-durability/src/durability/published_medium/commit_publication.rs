//! A live commit's distinct memory-only or medium-committed cutover.

use std::sync::Arc;

use worth_proof::DurablyCommitted;

use super::RelationalDurableCommitReadmission;
use super::RelationalDurableMediumBinding;
use worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::step_stop::RelationalDurableStepStop;

pub enum RelationalCommitPublication {
    MemoryOnly {
        positioned: Arc<PositionedCanonicalCommit>,
        original: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact,
    },
    Durable(DurablyCommitted<RelationalDurableCommitReadmission>),
}

impl RelationalCommitPublication {
    pub fn prepare_memory_frame(
        position: worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition,
        prepared: &crate::durability::canonical_commit_artifact::PreparedOriginalCanonicalPayload,
        medium: Option<&RelationalDurableMediumBinding>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact>,
        crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial,
    > {
        if medium.is_some() {
            return Ok(None);
        }
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_memory_publication(position,prepared,execution).map(Some)
    }
    /// Publish or retain the stream's own subject. A caller cannot substitute
    /// another admitted publication for the prepared route's commit.
    pub fn publish(
        positioned: Arc<PositionedCanonicalCommit>,
        roots: &worth_relational_contracts::internal_port::branch::target::RelationalBranchRootDescriptor,
        medium: Option<&RelationalDurableMediumBinding>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: Option<
            crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact,
        >,
    ) -> Result<Self, RelationalDurableStepStop> {
        match medium {
            Some(medium) => medium
                .publish(positioned, roots, execution)
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
    pub fn cutover<Output>(
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
