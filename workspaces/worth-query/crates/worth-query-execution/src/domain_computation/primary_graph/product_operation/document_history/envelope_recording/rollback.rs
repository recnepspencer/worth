//! A refused envelope leaves the document at its prior head with no pending record.
use super::super::{
    recording::{RecordingKind, WorthQueryRefusedCandidate},
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryReservation,
};
use crate::{
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryExpectedProductBranch,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Release the envelope's reservation and give up its private candidate.
    ///
    /// `candidate` is absent when the fork itself failed. With a phase the
    /// candidate closes as an owned head, so a refused close keeps its close or
    /// cleanup owner. Without a phase, because the request can no longer
    /// advance, the live candidate is kept unclosed. A candidate that did not
    /// close is parked in the reservation's prepaid head slot, where retention
    /// maintenance and final disposal close it. The record is always released,
    /// so a retry with the same key starts fresh.
    ///
    /// The reservation must be the one this document issued for this envelope.
    pub fn abandon_history_envelope(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: Option<WorthQueryExpectedProductBranch<'runtime, S>>,
    ) {
        let candidate =
            candidate.map(|candidate| WorthQueryRefusedCandidate::new(candidate).close(phase));
        if self
            .refuse(RecordingKind::Envelope, reserved, candidate)
            .is_err()
        {
            panic!("an envelope abandons only the record its own document reserved");
        }
    }
}
