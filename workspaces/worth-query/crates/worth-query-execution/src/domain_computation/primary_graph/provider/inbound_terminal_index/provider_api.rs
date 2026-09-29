//! Narrow provider facade over World-backed compact terminal evidence.

use worth_runtime_world::facade::{
    RuntimeWorldPerformedPublicationProtection, RuntimeWorldPublicationCursor,
    RuntimeWorldPublicationFrontier,
};

use super::{WorthQueryCanonicalInboundCompletion, WorthQueryInboundTerminalIndexDenial};
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryInboundTerminalOwnerResult,
};
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::PerformedInstalledTransportCompletion;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn mark_inbound_completion_publication_pending(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) {
        self.inbound_terminal_index.mark_pending(*correlation);
    }

    pub(in crate::domain_computation::primary_graph) fn clear_inbound_completion_publication_pending(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) {
        self.inbound_terminal_index.clear_pending(correlation);
    }

    pub(in crate::domain_computation::primary_graph) fn inbound_reconstruction_cursor(
        &self,
    ) -> (Option<RuntimeWorldPublicationCursor>, u64) {
        self.inbound_terminal_index.reconstruction_cursor()
    }

    pub(in crate::domain_computation::primary_graph) fn advance_inbound_reconstruction(
        &self,
        generation: u64,
        frontier: &RuntimeWorldPublicationFrontier,
        next: Option<RuntimeWorldPublicationCursor>,
        rows: impl IntoIterator<Item = WorthQueryCanonicalInboundCompletion>,
    ) -> Result<bool, WorthQueryInboundTerminalIndexDenial> {
        self.inbound_terminal_index
            .advance_reconstruction(generation, frontier, next, rows)
    }

    pub(in crate::domain_computation::primary_graph) fn abandon_inbound_reconstruction(&self) {
        self.inbound_terminal_index.abandon_reconstruction();
    }

    pub(in crate::domain_computation::primary_graph) fn retain_canonical_inbound_terminal(
        &self,
        terminal: &WorthQueryInboundTerminalOwnerResult,
        protection: RuntimeWorldPerformedPublicationProtection,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        self.inbound_terminal_index.retain(terminal, protection)
    }

    pub(in crate::domain_computation::primary_graph) fn retain_canonical_transport_terminal(
        &self,
        terminal: &PerformedInstalledTransportCompletion,
        protection: RuntimeWorldPerformedPublicationProtection,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        self.inbound_terminal_index
            .retain_transport(terminal, protection)
    }

    pub(in crate::domain_computation::primary_graph) fn lookup_completed_inbound(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Result<Option<WorthQueryCanonicalInboundCompletion>, WorthQueryInboundTerminalIndexDenial>
    {
        self.inbound_terminal_index.lookup(correlation)
    }

    pub(in crate::domain_computation::primary_graph) fn prune_completed_inbound(
        &self,
        correlations: &[ExternalEffectCorrelationIdentity],
        now_unix_seconds: u64,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        self.inbound_terminal_index
            .prune_expired(correlations, now_unix_seconds)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn destroy_completed_inbound_index_for_test(
        &self,
    ) {
        self.inbound_terminal_index.destroy_derived();
    }
}
