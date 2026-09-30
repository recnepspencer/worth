//! World-sealed terminal handoff before original outbox provenance release.

use worth_runtime_world::facade::RuntimeWorldPerformedPublicationProtection;

use super::{
    OutstandingDispatchInFlightLease, WorthQueryOutstandingInFlightDenial,
    WorthQueryPrimaryGraphProvider, WorthQueryTerminalDispatchReleaseDenial,
};
use crate::domain_computation::application_aftermath::WorthQueryInboundTerminalOwnerResult;
use crate::domain_computation::primary_graph::{
    PerformedInstalledTransportCompletion, WorthQueryCommittedDispatchOutboxObservation,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn begin_external_dispatch_in_flight(
        &self,
        original: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<Option<OutstandingDispatchInFlightLease>, WorthQueryOutstandingInFlightDenial> {
        if original.record().inbound().is_none() {
            return Ok(None);
        }
        self.outstanding_dispatch
            .begin_in_flight(original, self.receipt_basis_retention.clone())
            .map(Some)
    }

    pub(in crate::domain_computation::primary_graph) fn release_terminal_dispatch_provenance(
        &self,
        terminal: &WorthQueryInboundTerminalOwnerResult,
        protection: RuntimeWorldPerformedPublicationProtection,
    ) -> Result<(), WorthQueryTerminalDispatchReleaseDenial> {
        self.retain_canonical_inbound_terminal(terminal, protection)
            .map_err(|_| WorthQueryTerminalDispatchReleaseDenial::TerminalIndexUnavailable)?;
        let mut basis = self
            .receipt_basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.outstanding_dispatch
            .release_terminal(terminal, &mut basis)
    }

    pub(in crate::domain_computation::primary_graph) fn release_installed_transport_dispatch_provenance(
        &self,
        terminal: &PerformedInstalledTransportCompletion,
        protection: RuntimeWorldPerformedPublicationProtection,
    ) -> Result<(), WorthQueryTerminalDispatchReleaseDenial> {
        self.retain_canonical_transport_terminal(terminal, protection)
            .map_err(|_| WorthQueryTerminalDispatchReleaseDenial::TerminalIndexUnavailable)?;
        let mut basis = self
            .receipt_basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.outstanding_dispatch
            .release_transport_terminal(terminal, &mut basis)
    }
}
