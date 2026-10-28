//! The branch transition tail a recovering home hands to Relational: every
//! durable transition after the selected checkpoint's transition coverage,
//! in sequence, as untrusted artifact bytes. Store checks only its own
//! journal frame and the artifact frame; Relational readmits the transitions
//! and decides whether they continue the checkpoint.

use worth_relational::facade::durability::{
    RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactFrameRefusal,
};
use worth_store::physical_runtime::{
    OrderedJournalDenial, OrderedJournalReadFailure, ServingPhysicalRuntime,
};

/// The largest transition artifact one journal read admits. A transition
/// names branches and one head, so a larger record is refused, not
/// allocated.
pub(crate) const MAXIMUM_TRANSITION_BYTES: u64 = 1 << 20;

/// Why the recovered transition journal could not be handed to Relational.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalStoreTransitionJournalRefusal {
    /// The serving Store would not lend its transition journal.
    Unavailable(OrderedJournalDenial),
    /// The journal or one of its records could not be read or failed Store's
    /// own frame, linkage or digest checks.
    Unreadable {
        sequence: Option<u64>,
        failure: OrderedJournalReadFailure,
    },
    /// A journal record's payload is not an intact Relational transition
    /// artifact frame.
    ArtifactFrame {
        sequence: u64,
        refusal: RelationalBranchTransitionArtifactFrameRefusal,
    },
    /// The selected checkpoint covers transitions the journal does not hold.
    CheckpointAheadOfJournal { covered: u64, head: u64 },
}

type Refusal = RelationalStoreTransitionJournalRefusal;

/// Every transition artifact after `covered`, oldest first.
pub(crate) fn transition_tail(
    serving: &ServingPhysicalRuntime,
    covered: u64,
) -> Result<Vec<RelationalBranchTransitionArtifact>, Refusal> {
    let journal = serving.branch_transitions().map_err(Refusal::Unavailable)?;
    let head = journal
        .head()
        .map_err(|failure| Refusal::Unreadable {
            sequence: None,
            failure,
        })?
        .map_or(0, |entry| entry.position());
    if covered > head {
        return Err(Refusal::CheckpointAheadOfJournal { covered, head });
    }
    let mut tail = Vec::new();
    for sequence in covered + 1..=head {
        let payload = journal
            .read(sequence, MAXIMUM_TRANSITION_BYTES)
            .map_err(|failure| Refusal::Unreadable {
                sequence: Some(sequence),
                failure,
            })?;
        let artifact =
            RelationalBranchTransitionArtifact::from_untrusted_bytes(payload.into_bytes())
                .map_err(|refusal| Refusal::ArtifactFrame { sequence, refusal })?;
        tail.push(artifact);
    }
    Ok(tail)
}
