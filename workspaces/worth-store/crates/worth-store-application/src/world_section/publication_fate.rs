//! Only actual task terminals classify a submitted World journal publication.
use worth_foundational::facade::{CanonicalDigestId, WorldDurableJournal};
use worth_runtime_world::facade::{
    WorldDurableMediumDenial as Denial, WorldPublicationCompletion, WorldPublicationObservation,
};
use worth_store::physical_runtime::*;

pub(super) fn committed(committed: &OrderedJournalCommitted) -> WorldPublicationObservation {
    let source = committed.source();
    let journal = match source.journal().as_str() {
        "world.branches" => WorldDurableJournal::Branches,
        "world.occurrences" => WorldDurableJournal::Occurrences,
        "world.custody" => WorldDurableJournal::Custody,
        _ => {
            return WorldPublicationObservation::Finished(
                WorldPublicationCompletion::indeterminate(Denial::PublicationUnresolved),
            )
        }
    };
    let Some(position) = std::num::NonZeroU64::new(committed.entry().position()) else {
        return WorldPublicationObservation::EvidenceRequired;
    };
    WorldPublicationObservation::Finished(WorldPublicationCompletion::committed(
        journal,
        position,
        CanonicalDigestId::new(committed.entry().payload_sha256()),
    ))
}

pub(super) fn terminal(
    terminal: &OrderedJournalPublicationTerminal,
) -> WorldPublicationObservation {
    if terminal.published().is_some() {
        return WorldPublicationObservation::EvidenceRequired;
    }
    let detail = Denial::Storage(format!("{terminal:?}"));
    let fate = if terminal.failure().is_some() {
        WorldPublicationObservation::Finished(WorldPublicationCompletion::indeterminate(detail))
    } else {
        match terminal.physical_outcome() {
            Some(PhysicalMutationOutcome::ProvenNoEffect(_)) => {
                WorldPublicationObservation::Finished(WorldPublicationCompletion::proven_no_effect(
                    detail,
                ))
            }
            Some(PhysicalMutationOutcome::Completed(_))
            | Some(PhysicalMutationOutcome::Indeterminate(_))
            | None => WorldPublicationObservation::Finished(
                WorldPublicationCompletion::indeterminate(detail),
            ),
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
        WorldPublicationObservation::Finished(WorldPublicationCompletion::proven_no_effect(detail))
    } else {
        WorldPublicationObservation::Finished(WorldPublicationCompletion::indeterminate(detail))
    }
}
