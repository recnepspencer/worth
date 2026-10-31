//! One record batch submitted as one durable root decision, with its fate
//! classified.
//!
//! Every Store-owned door that writes records on an owner's behalf (the
//! ordered journals, anchored records and their members) publishes
//! through exactly this path: a built batch, a door-specific idempotency
//! material, platform durability and the ordinary durable append. The doors
//! differ only in what the batch holds and how they name the answer.

use worth_proof::TransitionOutcome;

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, IndeterminatePhysicalMutation, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordSubmission,
    ProvenNoEffectPhysicalMutation, RecordAppendBatch, RecordAppendDenial,
};

/// A built batch and how to submit it.
pub(in crate::physical_runtime) struct DurableRecordAppend {
    pub(in crate::physical_runtime) batch: RecordAppendBatch,
    /// Door-specific, domain-separated idempotency material.
    pub(in crate::physical_runtime) idempotency: [u8; 32],
    pub(in crate::physical_runtime) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime) deadline: PhysicalMutationDeadline,
}

/// A durable append that did not report its records. Only `Indeterminate`
/// and `CompletedWithoutRecords` may have had an effect.
pub(in crate::physical_runtime) enum DurableRecordAppendFailure {
    Batch(RecordAppendDenial),
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
    /// The mutation completed but did not report one record per batch entry.
    CompletedWithoutRecords,
}

pub(in crate::physical_runtime) enum PreparedDurableRecordAppend {
    Prepared(crate::physical_runtime::PreparedPhysicalMutation),
    Terminal(crate::physical_runtime::PhysicalMutationTerminalFact),
}

impl DurableRecordAppend {
    pub(in crate::physical_runtime) fn prepare_with_interruption(
        self,
        submission: &PhysicalRecordSubmission,
        interruption: Option<std::sync::Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> Result<PreparedDurableRecordAppend, DurableRecordAppendFailure> {
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(self.idempotency))
            .map_err(DurableRecordAppendFailure::Idempotency)?;
        let request = PhysicalMutationRequest::platform_durable(key, self.deadline)
            .with_shared_interruption(interruption);
        match submission
            .prepare_durable_append(self.batch, self.placement, request)
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                Ok(PreparedDurableRecordAppend::Prepared(prepared))
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(done)) => {
                Ok(PreparedDurableRecordAppend::Terminal(
                    crate::physical_runtime::PhysicalMutationTerminalFact::from_outcome(
                        PhysicalMutationOutcome::Completed(done),
                    ),
                ))
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(
                fate,
            )) => Ok(PreparedDurableRecordAppend::Terminal(
                crate::physical_runtime::PhysicalMutationTerminalFact::from_outcome(
                    PhysicalMutationOutcome::ProvenNoEffect(fate),
                ),
            )),
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
                Ok(PreparedDurableRecordAppend::Terminal(
                    crate::physical_runtime::PhysicalMutationTerminalFact::from_outcome(
                        PhysicalMutationOutcome::Indeterminate(fate),
                    ),
                ))
            }
            other => Err(DurableRecordAppendFailure::Preparation(other.into())),
        }
    }
}
