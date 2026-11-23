//! Every end of a recorded envelope returns the history document.
use super::super::*;
use crate::application_entry::mutation::envelope::{
    WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeOutcome,
    WorthQueryEditEnvelopeReceipt,
};

/// The envelope is this document's step. `Recorded` carries the live stage
/// results; `AlreadyRecorded` names the step an earlier call with the same key
/// and intent recorded, and ran nothing.
#[must_use = "retain or close the returned history document"]
pub enum WorthQueryHistoryEditEnvelopeOutcome<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    Recorded {
        document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        step: WorthQueryHistoryStep,
        receipt: WorthQueryEditEnvelopeReceipt,
    },
    AlreadyRecorded {
        document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        step: WorthQueryHistoryStep,
    },
}

impl<S, H, Payload, Outputs> std::fmt::Debug
    for WorthQueryHistoryEditEnvelopeOutcome<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, step) = match self {
            Self::Recorded { step, .. } => ("Recorded", step),
            Self::AlreadyRecorded { step, .. } => ("AlreadyRecorded", step),
        };
        f.debug_struct(name)
            .field("step", step)
            .finish_non_exhaustive()
    }
}

/// Why the envelope recorded nothing. The document is at its prior head with
/// no pending record, so the same key may be retried.
#[derive(Debug)]
pub enum WorthQueryHistoryEditEnvelopeDenial {
    /// No stage was authored.
    Empty,
    /// The document, its current read or its journal refused before any effect.
    History(WorthQueryApplicationHistoryDenial),
    /// The document refused this stage.
    Stage {
        ordinal: usize,
        denial: WorthQueryDocumentHistoryDenial,
    },
    /// The private candidate could not be forked from the document head.
    Candidate(WorthQueryDocumentHistoryCandidateDenial),
    /// The envelope ended without committing; the outcome names its stage.
    Envelope(WorthQueryEditEnvelopeOutcome),
    Execution(WorthQueryEditEnvelopeExecutionDenial),
    /// The stages committed on the private candidate, but the document head
    /// could not adopt it. The candidate was discarded with its commit.
    Adoption(WorthQueryDocumentHistoryDenial),
}

#[must_use = "retain or close the returned history document"]
pub struct WorthQueryHistoryEditEnvelopeFailure<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    pub(super) document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    pub(super) denial: WorthQueryHistoryEditEnvelopeDenial,
}

impl<'runtime, S, H, Payload, Outputs>
    WorthQueryHistoryEditEnvelopeFailure<'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        WorthQueryHistoryEditEnvelopeDenial,
    ) {
        (self.document, self.denial)
    }

    pub fn denial(&self) -> &WorthQueryHistoryEditEnvelopeDenial {
        &self.denial
    }
}

impl<S, H, Payload, Outputs> std::fmt::Debug
    for WorthQueryHistoryEditEnvelopeFailure<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryEditEnvelopeFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
