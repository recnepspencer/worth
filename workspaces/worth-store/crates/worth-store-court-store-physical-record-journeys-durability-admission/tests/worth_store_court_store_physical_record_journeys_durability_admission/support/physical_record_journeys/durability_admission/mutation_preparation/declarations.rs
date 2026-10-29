pub(crate) use {std::num::NonZeroU32};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit, worth_store_testkit_store::physical_record_journeys::durability_with_pending_limit, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationAdmissionDisposition, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDeferred, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalMutationPreparationFailure, worth_store::physical_runtime::PhysicalMutationPreparationRebindRequired, worth_store::physical_runtime::PhysicalMutationPreparationStale, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordStreamFailureKind, worth_store::physical_runtime::RecordWriteSource, worth_store::physical_runtime::RecordWriteSourceError};

pub(crate) fn request(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
) -> PhysicalMutationRequest {
    request_at(key, 1_000)
}

pub(crate) fn request_at(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
    deadline: u64,
) -> PhysicalMutationRequest {
    worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(deadline).expect("deadline is positive"),
        ),
    )
}

pub(crate) struct ShortSource {
    pub(crate) emitted: bool,
}

impl RecordWriteSource for ShortSource {
    fn declared_length(&self) -> u64 {
        4
    }

    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        if self.emitted {
            return Ok(0);
        }
        self.emitted = true;
        target[..2].copy_from_slice(b"ab");
        Ok(2)
    }
}

pub(crate) struct ExactSource {
    pub(crate) bytes: Vec<u8>,
    pub(crate) offset: usize,
}

impl ExactSource {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            offset: 0,
        }
    }
}

impl RecordWriteSource for ExactSource {
    fn declared_length(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        let count = target
            .len()
            .min(self.bytes.len().saturating_sub(self.offset))
            .min(3);
        target[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}