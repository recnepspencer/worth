use worth_proof::TransitionOutcome;

use crate::facade::RuntimeBridge;
use crate::input::envelope::BridgeCommittedPatchEnvelope;

use super::delivery::CorrespondenceDeliveryOutcome;
use super::delivery_preflight::{admit_envelope_source, preflight_graph_instance};
use super::semantic_delivery_match::match_envelope;
use super::{
    BridgeCorrespondenceAdmissionFailure, BridgeCorrespondenceDeferred,
    BridgeCorrespondenceDeliveryDenial, BridgeCorrespondenceRebindRequired,
    BridgeCorrespondenceStale, BridgeDeliveredCorrespondenceChangeSet,
    BridgeInstalledSemanticCorrespondence, BridgePreparedScopedSignalInvalidation,
    CorrespondenceDeliveryCounters, ProvenCorrespondenceTargets,
};

pub(crate) type BridgeCorrespondencePreparationOutcome = TransitionOutcome<
    BridgePreparedCorrespondenceDelivery,
    BridgeCorrespondenceDeliveryDenial,
    BridgeCorrespondenceDeferred,
    BridgeCorrespondenceStale,
    BridgeCorrespondenceRebindRequired,
    BridgeCorrespondenceAdmissionFailure,
>;

pub(crate) struct BridgePreparedCorrespondenceDelivery {
    pub(crate) counters: CorrespondenceDeliveryCounters,
    pub(crate) change_set: BridgeDeliveredCorrespondenceChangeSet,
    pub(crate) signal: Option<BridgePreparedScopedSignalInvalidation>,
    pub(crate) target_count: usize,
    pub(crate) node_fan_out: usize,
}

impl RuntimeBridge {
    pub(crate) fn prepare_installed_correspondence_envelope_to_targets(
        &self,
        correspondence: &BridgeInstalledSemanticCorrespondence,
        targets: &ProvenCorrespondenceTargets,
        graph_instance_id: u64,
        envelope: &BridgeCommittedPatchEnvelope,
        mut counters: CorrespondenceDeliveryCounters,
    ) -> BridgeCorrespondencePreparationOutcome {
        if let Err(denial) = admit_envelope_source(correspondence, envelope, counters) {
            return TransitionOutcome::Denied(denial);
        }
        if let Some(outcome) = preflight_graph_instance(self, correspondence, graph_instance_id) {
            return map_preflight(outcome);
        }
        let target_slice = targets.as_slice();
        if let Err(outcome) =
            self.admit_delivery_target_allocations(correspondence, target_slice, &mut counters)
        {
            return map_preflight(outcome);
        }
        let matched = match match_envelope(
            correspondence.ready.payload(),
            target_slice,
            envelope,
            counters,
        ) {
            Ok(matched) => matched,
            Err(denial) => return TransitionOutcome::Denied(denial),
        };
        let counters = matched.counters;
        let change_set = BridgeDeliveredCorrespondenceChangeSet::new(
            correspondence.basis().clone(),
            correspondence.dependency().clone(),
            envelope,
            matched.changes,
        );
        let signal = (counters.truth_targets_admitted != 0).then(|| {
            super::signal_execution::prepare_scoped_signal_invalidation_for_targets(
                correspondence,
                target_slice,
                &change_set,
            )
        });
        let node_fan_out = target_slice
            .iter()
            .map(|target| target.node)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        TransitionOutcome::Success(BridgePreparedCorrespondenceDelivery {
            counters,
            change_set,
            signal,
            target_count: target_slice.len(),
            node_fan_out,
        })
    }
}

fn map_preflight(outcome: CorrespondenceDeliveryOutcome) -> BridgeCorrespondencePreparationOutcome {
    match outcome {
        TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
        TransitionOutcome::Deferred(deferred) => TransitionOutcome::Deferred(deferred),
        TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
        TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
        TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
        TransitionOutcome::Success(_) => {
            unreachable!("delivery preflight cannot perform correspondence delivery")
        }
    }
}
