//! Relational's durable medium, implemented over the home's canonical commit
//! journal.
//!
//! Store never decodes the artifact. It persists the bytes at the artifact's
//! position under one durable root decision and reports the fate in
//! Relational's terms: durable, proven to have had no effect, or unknown.

use std::sync::Arc;

use worth_relational::facade::durability::{
    RelationalCanonicalCommitArtifact, RelationalDurableMedium, RelationalDurablePublication,
};
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, CanonicalCommitPublication, CanonicalCommitPublicationFailure,
    CanonicalCommitPublished, PhysicalMutationDeadline, ServingPhysicalRuntime,
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
        let journal = match self.serving.canonical_commits() {
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
        let publication = CanonicalCommitPublication::new(
            artifact.position().0,
            artifact.as_bytes().to_vec(),
            self.placement,
            deadline,
        );
        match journal.publish(publication) {
            Ok(
                CanonicalCommitPublished::Committed(_)
                | CanonicalCommitPublished::AlreadyCommitted(_),
            ) => RelationalDurablePublication::Committed,
            Err(failure) => publication_failure(failure),
        }
    }
}

/// Every journal failure either happened before any effect or leaves the fate
/// to recovery; nothing in between is reported as durable.
fn publication_failure(failure: CanonicalCommitPublicationFailure) -> RelationalDurablePublication {
    let detail = format!("{failure:?}");
    match failure {
        CanonicalCommitPublicationFailure::Indeterminate(_)
        | CanonicalCommitPublicationFailure::CompletedWithoutRecord => {
            RelationalDurablePublication::Indeterminate { detail }
        }
        CanonicalCommitPublicationFailure::EmptyPayload
        | CanonicalCommitPublicationFailure::PositionGap { .. }
        | CanonicalCommitPublicationFailure::PositionConflict(_)
        | CanonicalCommitPublicationFailure::Unresolved { .. }
        | CanonicalCommitPublicationFailure::Batch(_)
        | CanonicalCommitPublicationFailure::Idempotency(_)
        | CanonicalCommitPublicationFailure::Preparation(_)
        | CanonicalCommitPublicationFailure::ProvenNoEffect(_)
        | CanonicalCommitPublicationFailure::Read(_) => {
            RelationalDurablePublication::ProvenNoEffect { detail }
        }
        // A failure this build does not know cannot be proven harmless.
        _ => RelationalDurablePublication::Indeterminate { detail },
    }
}
