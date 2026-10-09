use worth_signal::facade::ResourceManagedQueueBinding;

use crate::execution_basis::authority::BridgeBoundExecutionBasisParts;
use crate::execution_basis::posture::{BridgeExecutionBasisPosture, BridgeManagedExecutionBasis};
use crate::execution_basis::{
    BridgeBoundExecutionBasis, BridgeExecutionBasisCounters,
    BridgeExecutionBasisReadmissionCommitted, BridgeExecutionBasisReadmissionYielded,
    BridgeManagedExecutionIntent, BridgeYieldedExecutionBasis,
};
use crate::source::AdmittedBridgeAsyncRequestIdentity;

use super::outcome::{
    BridgeExecutionBasisReadmissionCleanupOutcome, BridgeExecutionBasisReadmissionRecoveryRequired,
    BridgeProvisionalSignalAttempt,
};
use super::BridgeExecutionBasisReadmissionCounters;

#[must_use = "pending readmission must be committed through RuntimeBridge or explicitly aborted"]
pub struct BridgeExecutionBasisReadmissionPending {
    yielded: Option<BridgeYieldedExecutionBasis>,
    runtime_key: u64,
    fresh_intent: Option<BridgeManagedExecutionIntent>,
    managed_queue: Option<ResourceManagedQueueBinding>,
    provisional: Option<BridgeProvisionalSignalAttempt>,
    basis_counters: BridgeExecutionBasisCounters,
    counters: BridgeExecutionBasisReadmissionCounters,
}

impl BridgeExecutionBasisReadmissionPending {
    pub(super) fn new(
        yielded: BridgeYieldedExecutionBasis,
        runtime_key: u64,
        fresh_intent: BridgeManagedExecutionIntent,
        managed_queue: ResourceManagedQueueBinding,
        request: AdmittedBridgeAsyncRequestIdentity,
        reservation: crate::execution_basis::reservation::BridgeExecutionBasisReservation,
        basis_counters: BridgeExecutionBasisCounters,
        counters: BridgeExecutionBasisReadmissionCounters,
    ) -> Self {
        Self {
            yielded: Some(yielded),
            runtime_key,
            fresh_intent: Some(fresh_intent),
            managed_queue: Some(managed_queue),
            provisional: Some(BridgeProvisionalSignalAttempt::new(
                runtime_key,
                request,
                reservation,
            )),
            basis_counters,
            counters,
        }
    }

    pub fn fresh_request_identity(&self) -> &str {
        self.provisional
            .as_ref()
            .expect("pending readmission retains its Signal request")
            .request_identity()
    }

    pub fn step_contract(&self) -> &crate::execution_basis::BridgeManagedExecutionStepContract {
        self.yielded
            .as_ref()
            .expect("pending readmission retains its yielded Bridge authority")
            .step_contract()
    }

    pub const fn counters(&self) -> BridgeExecutionBasisReadmissionCounters {
        self.counters
    }

    pub(crate) const fn runtime_key(&self) -> u64 {
        self.runtime_key
    }

    pub fn abort(mut self) -> BridgeExecutionBasisReadmissionCleanupOutcome {
        self.counters.aborted();
        let yielded = self
            .yielded
            .take()
            .expect("readmission abort returns yielded authority once");
        let mut provisional = self
            .provisional
            .take()
            .expect("readmission abort owns provisional Signal authority");
        self.managed_queue.take();
        match provisional.cleanup() {
            Ok(()) => BridgeExecutionBasisReadmissionCleanupOutcome::Complete(
                BridgeExecutionBasisReadmissionYielded::new(yielded, self.counters),
            ),
            Err(detail) => BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(
                BridgeExecutionBasisReadmissionRecoveryRequired::new(
                    detail,
                    yielded,
                    provisional,
                    self.counters,
                ),
            ),
        }
    }

    pub(crate) fn commit(mut self) -> BridgeExecutionBasisReadmissionCommitted {
        self.counters.committed();
        let mut yielded = self
            .yielded
            .take()
            .expect("readmission commit consumes yielded Bridge authority");
        let observation = yielded.take_observation();
        let authoritative_source_profile = yielded.basis.authoritative_source_profile.clone();
        let step_contract = yielded.step_contract().clone();
        let (request, reservation) = self
            .provisional
            .take()
            .expect("readmission commit consumes provisional Signal authority")
            .into_parts();
        let basis = BridgeBoundExecutionBasis::new(BridgeBoundExecutionBasisParts {
            bridge_runtime_key: self.runtime_key,
            managed_intent: self
                .fresh_intent
                .take()
                .expect("readmission commit consumes fresh managed intent"),
            posture: BridgeExecutionBasisPosture::Managed(BridgeManagedExecutionBasis {
                step_contract,
                queue: self
                    .managed_queue
                    .take()
                    .expect("readmission commit consumes managed queue binding"),
                occupancy_width: 0,
            }),
            request,
            observation,
            authoritative_source_profile,
            reservation,
            counters: self.basis_counters.clone(),
        });
        BridgeExecutionBasisReadmissionCommitted::new(basis, self.counters)
    }
}

impl Drop for BridgeExecutionBasisReadmissionPending {
    fn drop(&mut self) {
        if let Some(provisional) = self.provisional.as_mut() {
            let _ = provisional.cleanup();
        }
    }
}
