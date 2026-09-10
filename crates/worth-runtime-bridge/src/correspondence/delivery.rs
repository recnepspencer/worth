use worth_proof::TransitionOutcome;
use worth_signal::facade::SignalGraph;

use crate::facade::RuntimeBridge;
use crate::input::envelope::{BridgeCommittedPatchEnvelope, BridgeProducerAuthorityKind};

use super::delivery_preflight::preflight;
use super::{
    BridgeCorrespondenceAdmissionFailure, BridgeCorrespondenceDeferred,
    BridgeCorrespondenceDeliveryDenial, BridgeCorrespondenceDeliveryReceipt,
    BridgeCorrespondenceDenialKind, BridgeCorrespondenceRebindRequired, BridgeCorrespondenceStale,
    BridgeInstalledSemanticCorrespondence, BridgePreparedCorrespondenceDelivery,
    CorrespondenceDeliveryCounters,
};

pub type CorrespondenceDeliveryOutcome = TransitionOutcome<
    BridgeCorrespondenceDeliveryReceipt,
    BridgeCorrespondenceDeliveryDenial,
    BridgeCorrespondenceDeferred,
    BridgeCorrespondenceStale,
    BridgeCorrespondenceRebindRequired,
    BridgeCorrespondenceAdmissionFailure,
>;

impl RuntimeBridge {
    pub(crate) fn deliver_installed_correspondence(
        &self,
        correspondence: &BridgeInstalledSemanticCorrespondence,
        graph: &mut SignalGraph,
        request: crate::adapter::RelationalCommittedPatchRequest,
    ) -> CorrespondenceDeliveryOutcome {
        if let Some(outcome) = preflight(self, correspondence, graph) {
            return outcome;
        }
        let requested_commit = request.commit_identity().clone();
        let envelope = match self.committed_patch_source.load_committed_patch(request) {
            Ok(envelope) => envelope,
            Err(_) => {
                return TransitionOutcome::Failed(
                    BridgeCorrespondenceAdmissionFailure::SourceLoadFailed,
                )
            }
        };
        if envelope.commit_identity() != &requested_commit {
            let mut counters = CorrespondenceDeliveryCounters::zero();
            counters.source_load_attempts = 1;
            counters.source_envelopes_loaded = 1;
            counters.failed_deliveries = 1;
            return TransitionOutcome::Denied(BridgeCorrespondenceDeliveryDenial::new(
                BridgeCorrespondenceDenialKind::CommittedPatchRequestMismatch,
                counters,
            ));
        }
        if envelope.producer_metadata().authority_kind()
            != BridgeProducerAuthorityKind::RegisteredAuthoritativeSource
        {
            let mut counters = CorrespondenceDeliveryCounters::zero();
            counters.source_load_attempts = 1;
            counters.source_envelopes_loaded = 1;
            counters.failed_deliveries = 1;
            return TransitionOutcome::Denied(BridgeCorrespondenceDeliveryDenial::new(
                BridgeCorrespondenceDenialKind::AuthoritativeSourceMismatch,
                counters,
            ));
        }
        let mut counters = CorrespondenceDeliveryCounters::zero();
        counters.source_load_attempts = 1;
        counters.source_envelopes_loaded = 1;
        self.deliver_installed_correspondence_envelope_with_counters(
            correspondence,
            graph,
            &envelope,
            counters,
        )
    }

    #[cfg(test)]
    pub(crate) fn deliver_installed_correspondence_envelope(
        &self,
        correspondence: &BridgeInstalledSemanticCorrespondence,
        graph: &mut SignalGraph,
        envelope: &BridgeCommittedPatchEnvelope,
    ) -> CorrespondenceDeliveryOutcome {
        self.deliver_installed_correspondence_envelope_with_counters(
            correspondence,
            graph,
            envelope,
            CorrespondenceDeliveryCounters::zero(),
        )
    }

    pub(crate) fn deliver_installed_correspondence_envelope_with_counters(
        &self,
        correspondence: &BridgeInstalledSemanticCorrespondence,
        graph: &mut SignalGraph,
        envelope: &BridgeCommittedPatchEnvelope,
        counters: CorrespondenceDeliveryCounters,
    ) -> CorrespondenceDeliveryOutcome {
        self.deliver_installed_correspondence_envelope_to_targets_with_counters(
            correspondence,
            &correspondence.targets,
            graph,
            envelope,
            counters,
        )
    }

    pub(crate) fn deliver_installed_correspondence_envelope_to_targets_with_counters(
        &self,
        correspondence: &BridgeInstalledSemanticCorrespondence,
        targets: &super::ProvenCorrespondenceTargets,
        graph: &mut SignalGraph,
        envelope: &BridgeCommittedPatchEnvelope,
        counters: CorrespondenceDeliveryCounters,
    ) -> CorrespondenceDeliveryOutcome {
        let graph_instance_id = graph.installed_graph_capability().graph_instance_id();
        let prepared = match self.prepare_installed_correspondence_envelope_to_targets(
            correspondence,
            targets,
            graph_instance_id,
            envelope,
            counters,
        ) {
            TransitionOutcome::Success(prepared) => prepared,
            TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
            TransitionOutcome::Deferred(deferred) => return TransitionOutcome::Deferred(deferred),
            TransitionOutcome::Stale(stale) => return TransitionOutcome::Stale(stale),
            TransitionOutcome::RebindRequired(rebind) => {
                return TransitionOutcome::RebindRequired(rebind)
            }
            TransitionOutcome::Failed(failure) => return TransitionOutcome::Failed(failure),
        };
        self.perform_prepared_delivery_on_raw_graph(graph, prepared)
    }

    fn perform_prepared_delivery_on_raw_graph(
        &self,
        graph: &mut SignalGraph,
        prepared: BridgePreparedCorrespondenceDelivery,
    ) -> CorrespondenceDeliveryOutcome {
        let BridgePreparedCorrespondenceDelivery {
            mut counters,
            change_set,
            signal,
            target_count,
            node_fan_out,
        } = prepared;
        let Some(prepared_signal) = signal else {
            return TransitionOutcome::Success(BridgeCorrespondenceDeliveryReceipt::new(
                counters, change_set, None,
            ));
        };
        let scoped_changes = match prepared_signal.admit_raw_graph(graph, &mut counters) {
            Ok(changes) => changes,
            Err(()) => {
                return TransitionOutcome::RebindRequired(
                    BridgeCorrespondenceRebindRequired::SignalGraphGeneration,
                )
            }
        };
        let worth_proof::TransitionOutcome::Success(admitted) =
            worth_signal::facade::apply_installed_scoped_changes(graph, scoped_changes)
        else {
            return TransitionOutcome::Failed(
                BridgeCorrespondenceAdmissionFailure::SignalMutationFailed,
            );
        };
        counters.signal_seeds_emitted = admitted.len();
        counters.node_fan_out = node_fan_out;
        counters.slots_touched = target_count;
        TransitionOutcome::Success(BridgeCorrespondenceDeliveryReceipt::new(
            counters,
            change_set,
            Some(prepared_signal),
        ))
    }

    pub(super) fn admit_delivery_target_allocations(
        &self,
        correspondence: &BridgeInstalledSemanticCorrespondence,
        targets: &[super::InstalledCorrespondenceTarget],
        counters: &mut CorrespondenceDeliveryCounters,
    ) -> Result<(), CorrespondenceDeliveryOutcome> {
        counters.allocation_registry_lock_attempts += 1;
        let allocation_registry = match self.correspondence_allocations.try_read() {
            Ok(registry) => registry,
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(TransitionOutcome::Deferred(
                    BridgeCorrespondenceDeferred::GraphMutationInProgress,
                ))
            }
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err(TransitionOutcome::Failed(
                    BridgeCorrespondenceAdmissionFailure::LockPoisoned,
                ))
            }
        };
        for target in targets {
            counters.allocation_source_set_checks += 1;
            if !allocation_registry.admits_source_set(target) {
                return Err(TransitionOutcome::RebindRequired(
                    BridgeCorrespondenceRebindRequired::AllocationSourceSet,
                ));
            }
        }
        drop(allocation_registry);
        let basis = correspondence.basis();
        for target in targets {
            counters.signal_basis_target_checks += 1;
            if target.signal_graph_instance_id != basis.signal_graph_instance_id
                || !basis.signal_partitions.contains(&target.partition)
            {
                return Err(TransitionOutcome::RebindRequired(
                    BridgeCorrespondenceRebindRequired::SignalGraphGeneration,
                ));
            }
        }
        Ok(())
    }
}
