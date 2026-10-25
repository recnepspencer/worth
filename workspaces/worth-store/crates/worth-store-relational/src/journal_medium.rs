//! Relational's durable medium, implemented over the home's two ordered
//! journals: canonical commits at their positions, branch transitions at
//! their sequences.
//!
//! Store never decodes an artifact. It persists the bytes at the artifact's
//! position or sequence under one durable root decision and reports the fate
//! in Relational's terms: durable, proven to have had no effect, or unknown.

use std::sync::Arc;

use worth_relational::facade::durability::{
    RelationalBranchTransitionArtifact, RelationalCanonicalCommitArtifact, RelationalDurableMedium,
    RelationalDurablePublication,
};
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, OrderedJournal, OrderedJournalDenial, OrderedJournalPublication,
    OrderedJournalPublicationFailure, OrderedJournalPublished, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

/// How long one journal publication may take before Store refuses it.
const PUBLICATION_DEADLINE_MILLIS: u64 = 30_000;

pub(crate) struct StoreJournalMedium {
    serving: Arc<ServingPhysicalRuntime>,
    placement: AdmittedRecordPlacementPolicy,
}

impl StoreJournalMedium {
    pub(crate) fn new(
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Self {
        Self { serving, placement }
    }

    /// Append `bytes` at `entry` of the borrowed journal.
    fn publish_entry(
        &self,
        journal: Result<OrderedJournal<'_>, OrderedJournalDenial>,
        entry: u64,
        bytes: &[u8],
    ) -> RelationalDurablePublication {
        let journal = match journal {
            Ok(journal) => journal,
            Err(denial) => {
                return RelationalDurablePublication::ProvenNoEffect {
                    detail: format!("journal unavailable: {denial:?}"),
                };
            }
        };
        let Some(deadline) =
            PhysicalMutationDeadline::after_milliseconds(PUBLICATION_DEADLINE_MILLIS)
        else {
            return RelationalDurablePublication::ProvenNoEffect {
                detail: "publication deadline is not representable".to_owned(),
            };
        };
        let publication =
            OrderedJournalPublication::new(entry, bytes.to_vec(), self.placement, deadline);
        match journal.publish(publication) {
            Ok(
                OrderedJournalPublished::Committed(_)
                | OrderedJournalPublished::AlreadyCommitted(_),
            ) => RelationalDurablePublication::Committed,
            Err(failure) => publication_failure(failure),
        }
    }
}

impl std::fmt::Debug for StoreJournalMedium {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StoreJournalMedium")
            .finish_non_exhaustive()
    }
}

impl RelationalDurableMedium for StoreJournalMedium {
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        self.publish_entry(
            self.serving.canonical_commits(),
            artifact.position().0,
            artifact.as_bytes(),
        )
    }

    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        self.publish_entry(
            self.serving.branch_transitions(),
            artifact.sequence().0,
            artifact.as_bytes(),
        )
    }
}

/// Every journal failure either happened before any effect or leaves the fate
/// to recovery; nothing in between is reported as durable.
fn publication_failure(failure: OrderedJournalPublicationFailure) -> RelationalDurablePublication {
    let detail = format!("{failure:?}");
    match failure {
        OrderedJournalPublicationFailure::Indeterminate(_)
        | OrderedJournalPublicationFailure::CompletedWithoutRecord => {
            RelationalDurablePublication::Indeterminate { detail }
        }
        OrderedJournalPublicationFailure::EmptyPayload
        | OrderedJournalPublicationFailure::PositionGap { .. }
        | OrderedJournalPublicationFailure::PositionConflict(_)
        | OrderedJournalPublicationFailure::AnchorTableFull
        | OrderedJournalPublicationFailure::Unresolved { .. }
        | OrderedJournalPublicationFailure::Batch(_)
        | OrderedJournalPublicationFailure::Idempotency(_)
        | OrderedJournalPublicationFailure::Preparation(_)
        | OrderedJournalPublicationFailure::ProvenNoEffect(_)
        | OrderedJournalPublicationFailure::Read(_) => {
            RelationalDurablePublication::ProvenNoEffect { detail }
        }
        // A failure this build does not know cannot be proven harmless.
        _ => RelationalDurablePublication::Indeterminate { detail },
    }
}
