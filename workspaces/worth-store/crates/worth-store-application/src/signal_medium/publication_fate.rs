//! Only actual task terminals classify a submitted Signal journal publication.
use worth_foundational::facade::SignalDurableJournal;
use worth_signal::facade::durability::{
    SignalDurableMediumDenial as Denial, SignalPublicationCompletion, SignalPublicationObservation,
};
use worth_store::physical_runtime::*;

pub(super) fn committed(committed: &OrderedJournalCommitted) -> SignalPublicationObservation {
    let source = committed.source();
    let journal = match source.journal().as_str() {
        "signal.definitions" => SignalDurableJournal::Definitions,
        "signal.advances" => SignalDurableJournal::Advances,
        _ => {
            return SignalPublicationObservation::Finished(
                SignalPublicationCompletion::indeterminate(Denial::PublicationUnresolved),
            )
        }
    };
    let Some(position) = std::num::NonZeroU64::new(committed.entry().position()) else {
        return SignalPublicationObservation::Pending;
    };
    SignalPublicationObservation::Finished(SignalPublicationCompletion::committed(
        journal,
        position,
        committed.entry().payload_sha256(),
    ))
}

pub(super) fn terminal(
    terminal: &OrderedJournalPublicationTerminal,
) -> SignalPublicationObservation {
    if terminal.published().is_some() {
        return SignalPublicationObservation::Pending;
    }
    let detail = Denial::Storage(format!("{terminal:?}"));

    if terminal.failure().is_some() {
        SignalPublicationObservation::Finished(SignalPublicationCompletion::indeterminate(detail))
    } else {
        match terminal.physical_outcome() {
            Some(PhysicalMutationOutcome::ProvenNoEffect(_)) => {
                SignalPublicationObservation::Finished(
                    SignalPublicationCompletion::proven_no_effect(detail),
                )
            }
            Some(PhysicalMutationOutcome::Completed(_))
            | Some(PhysicalMutationOutcome::Indeterminate(_))
            | None => SignalPublicationObservation::Finished(
                SignalPublicationCompletion::indeterminate(detail),
            ),
        }
    }
}

/// Start refusals precede submission. An unresolved or unrecognized owner
/// refusal cannot establish absence of an earlier publication of this position.
pub(super) fn start_failure(
    failure: &OrderedJournalOperationFailure,
) -> SignalPublicationObservation {
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
        SignalPublicationObservation::Finished(SignalPublicationCompletion::proven_no_effect(
            detail,
        ))
    } else {
        SignalPublicationObservation::Finished(SignalPublicationCompletion::indeterminate(detail))
    }
}
