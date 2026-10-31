//! Only actual task terminals classify a submitted World journal publication.
use worth_foundational::facade::DurablePublicationFate;
use worth_runtime_world::facade::{
    WorldDurableMediumDenial as Denial, WorldPublicationObservation,
};
use worth_store::physical_runtime::*;

pub(super) fn committed(_committed: &OrderedJournalCommitted) -> WorldPublicationObservation {
    WorldPublicationObservation::Finished(DurablePublicationFate::Committed)
}

pub(super) fn terminal(
    terminal: &OrderedJournalPublicationTerminal,
) -> WorldPublicationObservation {
    if terminal.failure().is_none() && terminal.published().is_some() {
        return WorldPublicationObservation::EvidenceRequired;
    }
    let detail = Denial::Storage(format!("{terminal:?}"));
    let fate = if terminal.failure().is_some() {
        DurablePublicationFate::Indeterminate { detail }
    } else {
        match terminal.physical_outcome() {
            Some(PhysicalMutationOutcome::ProvenNoEffect(_)) => {
                DurablePublicationFate::ProvenNoEffect { detail }
            }
            _ => DurablePublicationFate::Indeterminate { detail },
        }
    };
    WorldPublicationObservation::Finished(fate)
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
    WorldPublicationObservation::Finished(if no_effect {
        DurablePublicationFate::ProvenNoEffect { detail }
    } else {
        DurablePublicationFate::Indeterminate { detail }
    })
}
