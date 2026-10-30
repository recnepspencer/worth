pub(crate) use std::num::NonZeroU32;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationFailure,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordStreamFailureKind,
    worth_store::physical_runtime::RecordWriteSource,
    worth_store::physical_runtime::RecordWriteSourceError,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_group_limit,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

pub(crate) fn assert_source_failure(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    case: SourceFailureCase,
) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(
                [case.material; 32],
            ),
        )
        .unwrap();
    let before = serving.media_counters();
    let batch = worth_store::physical_runtime::RecordAppendBatch::builder()
        .push_source(case.source)
        .build()
        .unwrap();
    assert!(matches!(
        submission
            .prepare_durable_append(batch, placement, request(key.clone()))
            .into_raw(),
        worth_proof::TransitionOutcome::Failed(worth_store::physical_runtime::PhysicalMutationPreparationFailure::Stream(failure))
            if failure.kind() == case.expected
                && failure.completed_range() == (0..case.completed_bytes)
    ));
    assert_eq!(serving.media_counters(), before);
    assert!(matches!(
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                    b"valid retry".as_slice()
                ])
                .unwrap(),
                placement,
                request(key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(_)
        )
    ));
}

pub(crate) struct SourceFailureCase {
    pub(crate) source: MalformedSource,
    pub(crate) material: u8,
    pub(crate) expected: RecordStreamFailureKind,
    pub(crate) completed_bytes: u64,
}

pub(crate) fn request(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
) -> PhysicalMutationRequest {
    worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000)
                .expect("deadline is positive"),
        ),
    )
}

pub(crate) enum MalformedSource {
    Rejecting,
    Excess { first: bool },
    InvalidCount,
}

impl RecordWriteSource for MalformedSource {
    fn declared_length(&self) -> u64 {
        match self {
            Self::Excess { .. } => 2,
            Self::Rejecting | Self::InvalidCount => 1,
        }
    }

    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        match self {
            Self::Rejecting => {
                Err(worth_store::physical_runtime::RecordWriteSourceError::ProducerRejected)
            }
            Self::Excess { first } if *first => {
                *first = false;
                target.copy_from_slice(b"ab");
                Ok(2)
            }
            Self::Excess { .. } => {
                target[0] = b'c';
                Ok(1)
            }
            Self::InvalidCount => Ok(target.len() + 1),
        }
    }
}
