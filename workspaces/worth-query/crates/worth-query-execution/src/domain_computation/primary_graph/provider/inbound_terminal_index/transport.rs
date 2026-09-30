//! Compact installed-transport terminal proof from actual World Performed.

use worth_runtime_world::facade::RuntimeWorldPerformedPublicationProtection;

use super::{
    WorthQueryCanonicalInboundCompletion, WorthQueryCompletionProvenance,
    WorthQueryInboundTerminalIndex, WorthQueryInboundTerminalIndexDenial,
};
use crate::domain_computation::primary_graph::{
    InstalledTransportCompletion, PerformedInstalledTransportCompletion,
    WorthQueryInstalledTransportCompletionBinding,
};

impl WorthQueryCanonicalInboundCompletion {
    /// Match a completed transport report against the already World-performed
    /// terminal. An authenticated callback can win the race for the same
    /// effect. A transport winner's original attempt remains in terminal
    /// provenance, but a later physical attempt on that same outbox has a
    /// different attempt and observation identity by design.
    pub(in crate::domain_computation::primary_graph) fn matches_transport_observation(
        &self,
        evidence: &InstalledTransportCompletion,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> bool {
        let owner = evidence.committed();
        let record = owner.record();
        let dispatch = evidence.dispatch();
        if record.inbound() != Some(binding.contract())
            || !dispatch.matches_committed_owner(binding.runtime(), owner)
            || self.correlation != *record.correlation()
            || self.original_relational_commit != *owner.commit_reference()
            || self.original_world_commit
                != *owner.committed_product_publication().composite_commit()
            || self.original_incarnation
                != owner.committed_product_publication().product_incarnation()
            || self.operation != binding.operation()
            || self.audience != binding.audience()
            || self.source != binding.contract().source_identity()
            || self.correlation_family != record.correlation_family().as_str()
            || self.protocol_identity != *record.protocol_identity()
            || self.protocol_version != record.protocol_version()
            || self.payload != record.payload()
        {
            return false;
        }
        true
    }

    pub(super) fn seal_transport(terminal: &PerformedInstalledTransportCompletion) -> Option<Self> {
        let evidence = terminal.evidence();
        let original = evidence.committed();
        let dispatch = evidence.dispatch();
        let binding = terminal.binding();
        let record = original.record();
        let contract = record.inbound()?;
        if contract != binding.contract()
            || !dispatch.matches_committed_owner(binding.runtime(), original)
            || original
                .committed_product_publication()
                .product_incarnation()
                != terminal.original_incarnation()
            || original.committed_product_publication().relational_commit()
                != original.commit_reference()
        {
            return None;
        }
        let performed = terminal.publication().publication();
        let settlement = performed.component_results().relational_settlement()?;
        if performed
            .component_results()
            .relational_commit_result()
            .is_none()
            || performed.commit().identity() != performed.new_product_head().selected_commit()
            || performed.new_product_head().lifecycle_incarnation()
                != terminal.original_incarnation()
        {
            return None;
        }
        let observed = dispatch.causal_ladder().observation()?;
        Some(Self {
            correlation: *record.correlation(),
            operation: binding.operation().to_owned(),
            audience: binding.audience().to_owned(),
            source: contract.source_identity().to_owned(),
            key_epoch: None,
            message_identity: None,
            signed_meaning_digest: None,
            provenance: WorthQueryCompletionProvenance::InstalledTransportCompletion {
                attempt_identity: *dispatch.causal_ladder().attempt().identity().bytes(),
                observation_identity: *observed.identity().bytes(),
            },
            correlation_family: record.correlation_family().as_str().to_owned(),
            protocol_identity: record.protocol_identity().clone(),
            protocol_version: record.protocol_version(),
            payload: record.payload().to_vec(),
            expires_at_unix_seconds: None,
            original_incarnation: terminal.original_incarnation(),
            original_relational_commit: original.commit_reference().clone(),
            original_world_commit: original
                .committed_product_publication()
                .composite_commit()
                .clone(),
            completion_relational_commit: settlement.clone(),
            completion_world_commit: performed.commit().identity().clone(),
            completion_attempt: performed.attempt_identity().clone(),
        })
    }
}

impl WorthQueryInboundTerminalIndex {
    pub(super) fn retain_transport(
        &self,
        terminal: &PerformedInstalledTransportCompletion,
        protection: RuntimeWorldPerformedPublicationProtection,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        let sealed = WorthQueryCanonicalInboundCompletion::seal_transport(terminal)
            .ok_or(WorthQueryInboundTerminalIndexDenial::WorldPairMismatch)?;
        self.retain_sealed(sealed, protection)
    }
}
