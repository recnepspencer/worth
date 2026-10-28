//! The durable fate of one branch transition, decided from the transition
//! journal alone, in Relational's own terms: the artifact digest Relational
//! reported for it.

use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::{
    RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactDigest,
};
use worth_store::physical_runtime::{OrderedJournalResolution, ServingPhysicalRuntime};

use crate::recovered_transitions::{
    RelationalStoreTransitionJournalRefusal as Refusal, MAXIMUM_TRANSITION_BYTES,
};

/// What the transition journal says about one sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalStoreTransitionFate {
    /// The journal holds this sequence with this transition's artifact.
    Committed,
    /// The journal ends below this sequence; the transition did not become
    /// durable and may be performed again.
    NotCommitted { head: u64 },
    /// The sequence holds a different transition.
    Conflict,
    /// A publication of this sequence has an unknown fate in this process;
    /// reopening the home establishes it.
    Unresolved,
}

pub(super) fn resolve(
    serving: &ServingPhysicalRuntime,
    sequence: RelationalBranchTransitionSequence,
    digest: RelationalBranchTransitionArtifactDigest,
) -> Result<RelationalStoreTransitionFate, Refusal> {
    let journal = serving.branch_transitions().map_err(Refusal::Unavailable)?;
    let unreadable = |failure| Refusal::Unreadable {
        sequence: Some(sequence.0),
        failure,
    };
    // The journal decides presence; the transition's identity is checked on
    // the artifact itself.
    match journal.resolve(sequence.0, [0; 32]).map_err(unreadable)? {
        OrderedJournalResolution::NotCommitted { head } => {
            return Ok(RelationalStoreTransitionFate::NotCommitted { head });
        }
        OrderedJournalResolution::Unresolved => {
            return Ok(RelationalStoreTransitionFate::Unresolved);
        }
        OrderedJournalResolution::Committed(_) | OrderedJournalResolution::Conflict(_) => {}
    }
    let payload = journal
        .read(sequence.0, MAXIMUM_TRANSITION_BYTES)
        .map_err(unreadable)?;
    let artifact = RelationalBranchTransitionArtifact::from_untrusted_bytes(payload.into_bytes())
        .map_err(|refusal| Refusal::ArtifactFrame {
        sequence: sequence.0,
        refusal,
    })?;
    Ok(if artifact.payload_digest() == digest {
        RelationalStoreTransitionFate::Committed
    } else {
        RelationalStoreTransitionFate::Conflict
    })
}
