use crate::{WorthServerOperationPlanProof, WorthServerResponseEnvelope};

use super::{
    WorthServerOperationExecutionSlot, WorthServerOperationSchedulerCounters,
    WorthServerScheduledMutationResult, WorthServerSchedulerCancellationPosture,
    WorthServerSchedulerFailurePosture,
};

#[derive(Debug)]
pub struct WorthServerScheduledOperationOutcome {
    slot: WorthServerOperationExecutionSlot,
    shared_read_basis_identity: Option<String>,
    execution_digest: Option<
        Result<String, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>,
    >,
    mutation_result: Option<WorthServerScheduledMutationResult>,
    response_envelope: Option<WorthServerResponseEnvelope>,
    failure_posture: Option<WorthServerSchedulerFailurePosture>,
    cancellation_posture: Option<WorthServerSchedulerCancellationPosture>,
    scheduler_counters: WorthServerOperationSchedulerCounters,
}

impl WorthServerScheduledOperationOutcome {
    pub(crate) fn success(
        slot: WorthServerOperationExecutionSlot,
        shared_read_basis_identity: String,
        execution_digest: String,
        response_envelope: WorthServerResponseEnvelope,
        scheduler_counters: WorthServerOperationSchedulerCounters,
    ) -> Self {
        Self {
            slot,
            shared_read_basis_identity: Some(shared_read_basis_identity),
            execution_digest: Some(Ok(execution_digest)),
            mutation_result: None,
            response_envelope: Some(response_envelope),
            failure_posture: None,
            cancellation_posture: None,
            scheduler_counters,
        }
    }

    pub(crate) fn failed(
        slot: WorthServerOperationExecutionSlot,
        failure_posture: WorthServerSchedulerFailurePosture,
        scheduler_counters: WorthServerOperationSchedulerCounters,
    ) -> Self {
        Self {
            slot,
            shared_read_basis_identity: None,
            execution_digest: None,
            mutation_result: None,
            response_envelope: None,
            failure_posture: Some(failure_posture),
            cancellation_posture: None,
            scheduler_counters,
        }
    }

    pub(crate) fn cancelled(
        slot: WorthServerOperationExecutionSlot,
        cancellation_posture: WorthServerSchedulerCancellationPosture,
        scheduler_counters: WorthServerOperationSchedulerCounters,
    ) -> Self {
        Self {
            slot,
            shared_read_basis_identity: None,
            execution_digest: None,
            mutation_result: None,
            response_envelope: None,
            failure_posture: None,
            cancellation_posture: Some(cancellation_posture),
            scheduler_counters,
        }
    }

    pub(crate) fn success_without_counters(
        slot: WorthServerOperationExecutionSlot,
        shared_read_basis_identity: String,
        execution_digest: String,
        response_envelope: WorthServerResponseEnvelope,
    ) -> Self {
        Self::success(
            slot,
            shared_read_basis_identity,
            execution_digest,
            response_envelope,
            WorthServerOperationSchedulerCounters::default(),
        )
    }

    pub(crate) fn mutation_success(
        slot: WorthServerOperationExecutionSlot,
        mutation_result: WorthServerScheduledMutationResult,
        response_envelope: WorthServerResponseEnvelope,
        scheduler_counters: WorthServerOperationSchedulerCounters,
    ) -> Self {
        Self {
            slot,
            shared_read_basis_identity: None,
            execution_digest: Some(mutation_result.result_digest().map(str::to_string)),
            mutation_result: Some(mutation_result),
            response_envelope: Some(response_envelope),
            failure_posture: None,
            cancellation_posture: None,
            scheduler_counters,
        }
    }

    pub(crate) fn mutation_success_without_counters(
        slot: WorthServerOperationExecutionSlot,
        mutation_result: WorthServerScheduledMutationResult,
        response_envelope: WorthServerResponseEnvelope,
    ) -> Self {
        Self::mutation_success(
            slot,
            mutation_result,
            response_envelope,
            WorthServerOperationSchedulerCounters::default(),
        )
    }

    pub(crate) fn failed_without_counters(
        slot: WorthServerOperationExecutionSlot,
        failure_posture: WorthServerSchedulerFailurePosture,
    ) -> Self {
        Self::failed(
            slot,
            failure_posture,
            WorthServerOperationSchedulerCounters::default(),
        )
    }

    pub(crate) fn cancelled_without_counters(
        slot: WorthServerOperationExecutionSlot,
        cancellation_posture: WorthServerSchedulerCancellationPosture,
    ) -> Self {
        Self::cancelled(
            slot,
            cancellation_posture,
            WorthServerOperationSchedulerCounters::default(),
        )
    }

    pub(crate) fn ordinal(&self) -> usize {
        self.slot.ordinal()
    }

    pub(crate) fn attach_scheduler_counters(
        &mut self,
        scheduler_counters: WorthServerOperationSchedulerCounters,
    ) {
        self.scheduler_counters = scheduler_counters;
    }

    pub fn slot(&self) -> &WorthServerOperationExecutionSlot {
        &self.slot
    }

    pub fn plan_proof(&self) -> &WorthServerOperationPlanProof {
        self.slot.plan_proof()
    }

    pub fn shared_read_basis_identity(&self) -> Option<&str> {
        self.shared_read_basis_identity.as_deref()
    }

    pub fn response_envelope(&self) -> Option<&WorthServerResponseEnvelope> {
        self.response_envelope.as_ref()
    }

    pub fn execution_digest(
        &self,
    ) -> Result<
        Option<&str>,
        worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
    > {
        self.execution_digest
            .as_ref()
            .map(|digest| digest.as_deref().map_err(|cause| *cause))
            .transpose()
    }

    pub fn mutation_result(&self) -> Option<&WorthServerScheduledMutationResult> {
        self.mutation_result.as_ref()
    }

    pub fn failure_posture(&self) -> Option<&WorthServerSchedulerFailurePosture> {
        self.failure_posture.as_ref()
    }

    pub fn cancellation_posture(&self) -> Option<WorthServerSchedulerCancellationPosture> {
        self.cancellation_posture
    }

    pub fn scheduler_counters(&self) -> &WorthServerOperationSchedulerCounters {
        &self.scheduler_counters
    }
}
