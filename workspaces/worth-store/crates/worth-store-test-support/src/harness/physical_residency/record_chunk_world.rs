use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordChunkView,
    RecordAppendBatch, RecordAppendDenial, RecordByteLimit, RecordReadError, RecordReadLimits,
    RecordStreamFailure, ServingPhysicalRuntime,
};

use super::PhysicalResidencyStoreWorld;

#[derive(Debug)]
pub enum PhysicalResidencyRecordWorldFailure {
    ReadProtection(worth_store::physical_runtime::PhysicalReadProtectionDenial),
    EmptyPayload,
    PayloadTooLarge,
    Batch(RecordAppendDenial),
    IdempotencyAdmission,
    MutationPreparation,
    MutationExecution,
    Read(Box<RecordReadError>),
    Stream(RecordStreamFailure),
    MissingChunk,
}

impl PhysicalResidencyStoreWorld {
    pub fn with_record_chunk<R>(
        &self,
        payload: &[u8],
        run: impl FnOnce(&ServingPhysicalRuntime, PhysicalRecordChunkView<'_>) -> R,
    ) -> Result<R, PhysicalResidencyRecordWorldFailure> {
        let width = u32::try_from(payload.len())
            .map_err(|_| PhysicalResidencyRecordWorldFailure::PayloadTooLarge)?;
        let limit =
            RecordByteLimit::new(width).ok_or(PhysicalResidencyRecordWorldFailure::EmptyPayload)?;
        let batch = RecordAppendBatch::try_from_iter([payload])
            .map_err(PhysicalResidencyRecordWorldFailure::Batch)?;
        let submission = self.serving().record_submission();
        let material = <[u8; 32]>::from(Sha256::digest(payload));
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
            .map_err(|_| PhysicalResidencyRecordWorldFailure::IdempotencyAdmission)?;
        let request = PhysicalMutationRequest::platform_durable(
            key,
            PhysicalMutationDeadline::after_milliseconds(1_000)
                .expect("fixture mutation deadline is nonzero"),
        );
        let prepared = match submission
            .prepare_durable_append(batch, self.placement, request)
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                prepared
            }
            _ => return Err(PhysicalResidencyRecordWorldFailure::MutationPreparation),
        };
        let acknowledgment = match prepared.execute() {
            PhysicalMutationOutcome::Completed(completed) => completed.into_acknowledgment(),
            PhysicalMutationOutcome::ProvenNoEffect(_)
            | PhysicalMutationOutcome::Indeterminate(_) => {
                return Err(PhysicalResidencyRecordWorldFailure::MutationExecution)
            }
        };
        let mut session = self
            .serving()
            .records()
            .map_err(PhysicalResidencyRecordWorldFailure::ReadProtection)?
            .open(
                acknowledgment
                    .record_ids()
                    .next()
                    .expect("one completed record mutation produces one identity"),
                RecordReadLimits::new(limit),
            )
            .map_err(|error| PhysicalResidencyRecordWorldFailure::Read(Box::new(error)))?;
        let chunk = session
            .next_chunk()
            .map_err(PhysicalResidencyRecordWorldFailure::Stream)?
            .ok_or(PhysicalResidencyRecordWorldFailure::MissingChunk)?;
        Ok(run(self.serving(), chunk))
    }
}
