//! A refused envelope leaves the document at its prior head with no pending record.
use super::super::{
    head_custody::HistoryHeadCustody, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryReservation,
};
use crate::{
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryExpectedProductBranch,
        WorthQueryOwnedProductHeadCloseFailure,
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
        assert!(
            self.validate_reservation(&reserved).is_ok(),
            "an envelope abandons only the record its own document reserved"
        );
        let unclosed = match (candidate.map(|candidate| candidate.into_owned_head()), phase) {
            (None, _) => None,
            (Some(head), None) => Some(HistoryHeadCustody::Live(head)),
            (Some(head), Some(phase)) => match head.close(phase) {
                Ok(_) => None,
                Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial }) => {
                    Some(HistoryHeadCustody::Refused { head, denial })
                }
                Err(WorthQueryOwnedProductHeadCloseFailure::Retired { branch, failure }) => {
                    Some(HistoryHeadCustody::Retired { branch, failure })
                }
            },
        };
        let mut selected = self.selection.lock();
        match unclosed {
            None => selected.journal.cancel(reserved.record),
            Some(custody) => selected.journal.cancel_parking(reserved.record, custody),
        }
    }
}
