use std::num::NonZeroU32;

use super::super::{configuration, durability_with_group_limit, media, success};
use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationFailure, PhysicalMutationPreparationSuccess,
    PhysicalMutationRequest, PhysicalRecordInitialization, RecordAppendBatch,
    RecordStreamFailureKind, RecordWriteSource, RecordWriteSourceError, ServingPhysicalRuntime,
};

#[test]
fn malformed_stream_contracts_fail_exactly_without_effect_or_idempotency_binding() {
    let parent = tempfile::tempdir().unwrap();
    let media_owner = media(&parent.path().join("store"));
    let policy = durability_with_group_limit(&media_owner, NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media_owner.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );

    assert_source_failure(
        &serving,
        placement,
        SourceFailureCase {
            source: MalformedSource::Rejecting,
            material: 51,
            expected: RecordStreamFailureKind::ProducerRejected,
            completed_bytes: 0,
        },
    );
    assert_source_failure(
        &serving,
        placement,
        SourceFailureCase {
            source: MalformedSource::Excess { first: true },
            material: 52,
            expected: RecordStreamFailureKind::SourceExceededDeclaredLength,
            completed_bytes: 2,
        },
    );
    assert_source_failure(
        &serving,
        placement,
        SourceFailureCase {
            source: MalformedSource::InvalidCount,
            material: 53,
            expected: RecordStreamFailureKind::InvalidTransferCount,
            completed_bytes: 0,
        },
    );
    serving.close();
}

fn assert_source_failure(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    case: SourceFailureCase,
) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
            [case.material; 32],
        ))
        .unwrap();
    let before = serving.media_counters();
    let batch = RecordAppendBatch::builder()
        .push_source(case.source)
        .build()
        .unwrap();
    assert!(matches!(
        submission
            .prepare_durable_append(batch, placement, request(key.clone()))
            .into_raw(),
        TransitionOutcome::Failed(PhysicalMutationPreparationFailure::Stream(failure))
            if failure.kind() == case.expected
                && failure.completed_range() == (0..case.completed_bytes)
    ));
    assert_eq!(serving.media_counters(), before);
    assert!(matches!(
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"valid retry".as_slice()]).unwrap(),
                placement,
                request(key),
            )
            .into_raw(),
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(_))
    ));
}

struct SourceFailureCase {
    source: MalformedSource,
    material: u8,
    expected: RecordStreamFailureKind,
    completed_bytes: u64,
}

fn request(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
) -> PhysicalMutationRequest {
    PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::at(
            TemporalDuration::temporal_duration(1_000).expect("deadline is positive"),
        ),
    )
}

enum MalformedSource {
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
            Self::Rejecting => Err(RecordWriteSourceError::ProducerRejected),
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
