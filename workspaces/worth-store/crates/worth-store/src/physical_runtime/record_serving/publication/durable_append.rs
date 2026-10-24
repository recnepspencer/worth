//! One record batch submitted as one durable root decision, with its fate
//! classified.
//!
//! Every Store-owned door that writes records on an owner's behalf (the
//! canonical commit journal, anchored records and their members) publishes
//! through exactly this path: a built batch, a door-specific idempotency
//! material, platform durability and the ordinary durable append. The doors
//! differ only in what the batch holds and how they name the answer.

use worth_proof::TransitionOutcome;
use worth_store_physical_format::PersistedRecordIdentity;

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

impl DurableRecordAppendFailure {
    /// Whether the append may be durable, so its door must not assume it
    /// had no effect until recovery establishes the fate.
    pub(in crate::physical_runtime) const fn leaves_fate_unresolved(&self) -> bool {
        matches!(self, Self::Indeterminate(_) | Self::CompletedWithoutRecords)
    }
}

impl DurableRecordAppend {
    /// Issues the idempotency key, prepares and executes the durable append,
    /// and returns the persisted records in batch order.
    pub(in crate::physical_runtime) fn submit(
        self,
        submission: &PhysicalRecordSubmission,
    ) -> Result<Vec<PersistedRecordIdentity>, DurableRecordAppendFailure> {
        type Failure = DurableRecordAppendFailure;
        let records = self.batch.records.len();
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(self.idempotency))
            .map_err(Failure::Idempotency)?;
        let request = PhysicalMutationRequest::platform_durable(key, self.deadline);
        let completed = match submission
            .prepare_durable_append(self.batch, self.placement, request)
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                match prepared.execute() {
                    PhysicalMutationOutcome::Completed(completed) => completed,
                    PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                        return Err(Failure::ProvenNoEffect(fate))
                    }
                    PhysicalMutationOutcome::Indeterminate(fate) => {
                        return Err(Failure::Indeterminate(fate))
                    }
                }
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(done)) => done,
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(
                fate,
            )) => return Err(Failure::ProvenNoEffect(fate)),
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
                return Err(Failure::Indeterminate(fate))
            }
            other => return Err(Failure::Preparation(other.into())),
        };
        let persisted = completed.persisted_records();
        if persisted.len() != records {
            return Err(Failure::CompletedWithoutRecords);
        }
        Ok(persisted.to_vec())
    }
}
