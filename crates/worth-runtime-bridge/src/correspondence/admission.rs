use worth_proof::TransitionOutcome;
use worth_signal::facade::SignalGraph;

use crate::facade::RuntimeBridge;

use super::{
    BridgeCorrespondenceAdmissionFailure, BridgeCorrespondenceDeferred, BridgeCorrespondenceDenial,
    BridgeCorrespondenceDenialKind, BridgeCorrespondenceRebindRequired, BridgeCorrespondenceStale,
    BridgeInstalledSemanticCorrespondence, BridgeSemanticDependencyCandidate,
    CorrespondenceAdmissionCounters,
};

pub type CorrespondenceAdmissionOutcome = TransitionOutcome<
    BridgeInstalledSemanticCorrespondence,
    BridgeCorrespondenceDenial,
    BridgeCorrespondenceDeferred,
    BridgeCorrespondenceStale,
    BridgeCorrespondenceRebindRequired,
    BridgeCorrespondenceAdmissionFailure,
>;

impl RuntimeBridge {
    pub(crate) fn install_semantic_correspondence(
        &self,
        dependency: BridgeSemanticDependencyCandidate,
        graph: &SignalGraph,
    ) -> CorrespondenceAdmissionOutcome {
        let resolved = match super::resolution::resolve(self, dependency, graph) {
            Ok(resolved) => resolved,
            Err(outcome) => return outcome,
        };
        let mapped = match super::target_mapping::map_targets(self, resolved) {
            Ok(mapped) => mapped,
            Err(outcome) => return outcome,
        };
        let allocated = match super::target_allocation::allocate_targets(self, mapped) {
            Ok(allocated) => allocated,
            Err(outcome) => return outcome,
        };
        super::signal_admission::admit_signal_targets(allocated, graph)
    }

    pub fn rebuild_correspondence_allocation_index(
        &self,
    ) -> Result<super::BridgeCorrespondenceRebuildReport, BridgeCorrespondenceAdmissionFailure>
    {
        let mut registry = self
            .correspondence_allocations
            .write()
            .map_err(|_| BridgeCorrespondenceAdmissionFailure::LockPoisoned)?;
        let rebuilt = registry.reconstruct_derived_indexes();
        *registry = rebuilt;
        let exact_index_parity = registry.reconstruct_derived_indexes() == *registry;
        Ok(super::BridgeCorrespondenceRebuildReport::new(
            self.semantic_dependency_registry.admitted_count(),
            registry.authoritative_records.len(),
            registry.owners.len(),
            self.semantic_dependency_registry.rebuild_has_exact_parity(),
            self.aspect_registry.rebuilt_id_index_has_exact_parity()
                && self
                    .aspect_registry
                    .rebuilt_semantic_index_has_exact_parity(),
            exact_index_parity,
        ))
    }
}

pub(super) fn denied(
    kind: BridgeCorrespondenceDenialKind,
    mut counters: CorrespondenceAdmissionCounters,
) -> CorrespondenceAdmissionOutcome {
    counters.failed_admissions += 1;
    TransitionOutcome::Denied(BridgeCorrespondenceDenial::new(kind, counters))
}
