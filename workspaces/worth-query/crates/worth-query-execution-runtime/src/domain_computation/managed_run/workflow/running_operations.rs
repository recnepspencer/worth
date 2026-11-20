use worth_runtime_bridge::facade::{
    BridgeManagedQueueAdmission, BridgeManagedQueueFailure, BridgeManagedQueueMutationCounters,
};

use super::WorthQueryRunningWorkflowRun;
use crate::domain_computation::provider_session::graph_provider::bounded_step::{
    WorthQueryGraphProviderMemoryArena, WorthQueryGraphProviderMemorySnapshot,
};
use crate::domain_computation::{
    WorthQueryGraphProviderStepReport, WorthQueryProviderExecutionReleaseEvidence,
};

impl WorthQueryRunningWorkflowRun {
    pub fn release_or_retain_queue_occupancy(
        &mut self,
        occupancy: worth_runtime_bridge::facade::BridgeManagedQueueOccupancy,
    ) -> bool {
        self.affinity
            .provider_work_mut()
            .release_or_retain_queue_occupancy(&mut self.bridge_basis, occupancy)
    }

    pub fn begin_provider_step_call(&mut self) {
        self.affinity.provider_work_mut().begin_step_call();
    }

    pub fn complete_provider_step_call(&mut self) {
        self.affinity.provider_work_mut().complete_step_call();
    }

    pub fn interrupt_provider_step_call(&mut self) {
        self.affinity.provider_work_mut().interrupt_step_call();
    }

    pub fn abandon_provider_step_call(&mut self) {
        self.affinity.provider_work_mut().abandon();
    }

    pub fn record_provider_step_attempt(&mut self) {
        self.affinity
            .provider_work_mut()
            .record_provider_step_attempt();
    }

    pub fn record_provider_step_admission(
        &mut self,
        counters: super::super::provider_step_admission::WorthQueryProviderStepAdmissionCounters,
    ) {
        self.affinity
            .provider_work_mut()
            .record_provider_step_admission(counters);
    }

    pub fn record_provider_step_report(
        &mut self,
        report: &WorthQueryGraphProviderStepReport,
    ) {
        self.affinity.provider_work_mut().admit_step(report);
        self.affinity
            .provider_work_mut()
            .settle_artifacts(self.provider_artifact_occurrences.snapshot());
    }

    pub fn record_safe_point(
        &mut self,
        observation: &super::super::WorthQueryManagedSafePointObservation,
    ) {
        self.affinity
            .provider_work_mut()
            .record_safe_point(observation);
    }

    pub fn enqueue_provider_output(
        &mut self,
        width: u64,
    ) -> Result<BridgeManagedQueueAdmission, BridgeManagedQueueFailure> {
        self.bridge_basis.enqueue_managed_queue(width)
    }

    pub fn record_queue_mutation(
        &mut self,
        counters: BridgeManagedQueueMutationCounters,
    ) {
        self.affinity
            .provider_work_mut()
            .record_queue_mutation(counters);
    }

    pub fn release_projection_bytes(
        &mut self,
        retained_bytes: usize,
    ) -> bool {
        self.affinity
            .provider_work_mut()
            .release_projection_bytes(retained_bytes)
    }

    pub fn transfer_projection_to_output(
        &mut self,
        projection_bytes: usize,
        additional_bytes: usize,
    ) -> bool {
        self.affinity
            .provider_work_mut()
            .transfer_projection_to_output(projection_bytes, additional_bytes)
    }

    pub fn transfer_output_to_receipt(
        &mut self,
        envelope_bytes: usize,
        total_output_bytes: usize,
    ) -> bool {
        self.affinity
            .provider_work_mut()
            .transfer_output_to_receipt(envelope_bytes, total_output_bytes)
    }

    pub fn release_output_bytes(
        &mut self,
        retained_bytes: usize,
    ) -> bool {
        self.affinity
            .provider_work_mut()
            .release_output_bytes(retained_bytes)
    }

    pub fn record_provider_execution_release(
        &mut self,
        evidence: &WorthQueryProviderExecutionReleaseEvidence,
    ) {
        self.affinity
            .provider_work_mut()
            .record_provider_execution_release(evidence);
    }

    pub fn observe_active_provider_memory(
        &mut self,
        memory: WorthQueryGraphProviderMemorySnapshot,
    ) {
        self.affinity
            .provider_work_mut()
            .observe_active_provider_memory(memory);
    }

    pub fn retain_provider_memory(
        &mut self,
        memory: WorthQueryGraphProviderMemoryArena,
    ) {
        self.affinity
            .provider_work_mut()
            .retain_provider_memory(memory);
    }
}
