//! One record that advances one root anchor, submitted as one durable root
//! decision, with its fate classified.
//!
//! Every Store-owned anchored door (the canonical commit journal and the
//! anchored-record door) publishes exactly this shape: a single framed record,
//! the anchor advance naming it and expecting the generation the door observed,
//! a door-specific idempotency material, and the ordinary durable append path.
//! The doors differ only in their frames, their admission checks and how they
//! name the answer.

use worth_proof::TransitionOutcome;
use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchorName};

use super::batch::RecordBatchAnchorAdvance;
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, IndeterminatePhysicalMutation, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordSubmission,
    ProvenNoEffectPhysicalMutation, RecordAppendBatch, RecordAppendDenial,
};

/// A framed record and the anchor movement it requests.
pub(in crate::physical_runtime) struct AnchoredRecordAppend {
    pub(in crate::physical_runtime) record: Vec<u8>,
    pub(in crate::physical_runtime) anchor: RecordAnchorName,
    /// The anchor generation the door observed; zero when absent.
    pub(in crate::physical_runtime) expected_generation: u64,
    /// Door-specific, domain-separated idempotency material.
    pub(in crate::physical_runtime) idempotency: [u8; 32],
    pub(in crate::physical_runtime) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime) deadline: PhysicalMutationDeadline,
}

/// An anchored append that did not report its one durable record. Only
/// `Indeterminate` and `CompletedWithoutRecord` may have had an effect.
pub(in crate::physical_runtime) enum AnchoredRecordAppendFailure {
    Batch(RecordAppendDenial),
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
    CompletedWithoutRecord,
}

impl AnchoredRecordAppendFailure {
    /// Whether the append may be durable, so its door must refuse further
    /// publication until recovery establishes the fate.
    pub(in crate::physical_runtime) const fn leaves_fate_unresolved(&self) -> bool {
        matches!(self, Self::Indeterminate(_) | Self::CompletedWithoutRecord)
    }
}

impl AnchoredRecordAppend {
    /// Builds the one-record batch, issues its idempotency key, prepares and
    /// executes the durable append, and returns the persisted record.
    pub(in crate::physical_runtime) fn submit(
        self,
        submission: &PhysicalRecordSubmission,
    ) -> Result<PersistedRecordIdentity, AnchoredRecordAppendFailure> {
        type Failure = AnchoredRecordAppendFailure;
        let batch = RecordAppendBatch::builder()
            .push_owned(self.record)
            .build()
            .and_then(|batch| {
                batch.with_anchor_advance(RecordBatchAnchorAdvance {
                    name: self.anchor,
                    expected_generation: self.expected_generation,
                    record_index: 0,
                })
            })
            .map_err(Failure::Batch)?;
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(self.idempotency))
            .map_err(Failure::Idempotency)?;
        let request = PhysicalMutationRequest::platform_durable(key, self.deadline);
        let completed = match submission
            .prepare_durable_append(batch, self.placement, request)
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
        match completed.persisted_records() {
            [record] => Ok(*record),
            _ => Err(Failure::CompletedWithoutRecord),
        }
    }
}
