//! A live commit's distinct memory-only or medium-committed cutover.

use std::sync::Arc;

use worth_proof::DurablyCommitted;

use super::{
    RelationalDurableColdSource, RelationalDurableCommitReadmission,
    RelationalDurableMediumBinding, RelationalDurableStepStop,
};
use crate::history::data::PositionedCanonicalCommit;
use crate::history::RelationalCommitIndexEntry;

pub(crate) enum RelationalCommitPublication {
    MemoryOnly(Arc<PositionedCanonicalCommit>),
    Durable(DurablyCommitted<RelationalDurableCommitReadmission>),
}

impl RelationalCommitPublication {
    /// Publish or retain the stream's own subject. A caller cannot substitute
    /// another admitted publication for the prepared route's commit.
    pub(crate) fn publish(
        positioned: Arc<PositionedCanonicalCommit>,
        roots: &crate::branch::RelationalBranchRootDescriptor,
        medium: Option<&RelationalDurableMediumBinding>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, RelationalDurableStepStop> {
        match medium {
            Some(medium) => medium.publish(positioned, roots, policy).map(Self::Durable),
            None => Ok(Self::MemoryOnly(positioned)),
        }
    }

    /// Apply the exact admitted subject once. Only the durable branch supplies
    /// journal readmission, and it spends its proof around the live cutover.
    pub(crate) fn cutover<Output>(
        self,
        apply: impl FnOnce(
            Arc<PositionedCanonicalCommit>,
            Option<(
                RelationalCommitIndexEntry,
                Arc<dyn RelationalDurableColdSource>,
            )>,
        ) -> Output,
    ) -> Output {
        match self {
            Self::MemoryOnly(positioned) => apply(positioned, None),
            Self::Durable(committed) => committed.consume(|publication| {
                let (positioned, entry, source) = publication.into_parts();
                apply(positioned, Some((entry, source)))
            }),
        }
    }
}
