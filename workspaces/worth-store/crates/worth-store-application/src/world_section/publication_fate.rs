//! Only actual task terminals classify a submitted World journal publication.
use worth_foundational::facade::{CanonicalDigestId, WorldDurableJournal};
use worth_runtime_world::facade::{
    WorldDurableMediumDenial as Denial, WorldPublicationObservation,
};
use worth_store::physical_runtime::*;

pub(super) fn committed(committed: &OrderedJournalCommitted) -> WorldPublicationObservation {
    let source = committed.source();
    let journal = match source.journal().as_str() {
        "world.branches" => WorldDurableJournal::Branches,
        "world.occurrences" => WorldDurableJournal::Occurrences,
        "world.custody" => WorldDurableJournal::Custody,
        _ => {
            return WorldPublicationObservation::Indeterminate {
                detail: Denial::PublicationUnresolved,
            }
        }
    };
    WorldPublicationObservation::Committed {
        journal,
        position: committed.entry().position(),
        digest: CanonicalDigestId::new(committed.entry().payload_sha256()),
    }
}

pub(super) fn terminal(
    terminal: &OrderedJournalPublicationTerminal,
) -> WorldPublicationObservation {
    if terminal.published().is_some() {
        return WorldPublicationObservation::EvidenceRequired;
    }
    let detail = Denial::Storage(format!("{terminal:?}"));
    let fate = if terminal.failure().is_some() {
        WorldPublicationObservation::Indeterminate { detail }
    } else {
        match terminal.physical_outcome() {
            Some(PhysicalMutationOutcome::ProvenNoEffect(_)) => {
                WorldPublicationObservation::ProvenNoEffect { detail }
            }
            _ => WorldPublicationObservation::Indeterminate { detail },
        }
    };
    fate
}

/// Start refusals precede submission. An unresolved or unrecognized owner
/// refusal cannot establish absence of an earlier publication of this position.
pub(super) fn start_failure(
    failure: &OrderedJournalOperationFailure,
) -> WorldPublicationObservation {
    let detail = Denial::Storage(format!("{failure:?}"));
    use OrderedJournalOperationFailure as O;
    use OrderedJournalPublicationFailure as F;
    let no_effect = matches!(
        failure,
        O::OriginalRequestStopped(_)
            | O::Busy
            | O::Stopped
            | O::Publication(
                F::EmptyPayload
                    | F::PositionGap { .. }
                    | F::PositionConflict(_)
                    | F::AnchorTableFull
                    | F::Batch(_)
                    | F::Idempotency(_)
                    | F::Preparation(_)
                    | F::ProvenNoEffect(_)
                    | F::Read(_)
            )
    );
    if no_effect {
        WorldPublicationObservation::ProvenNoEffect { detail }
    } else {
        WorldPublicationObservation::Indeterminate { detail }
    }
}
