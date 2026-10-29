#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::stream_failure::*;
use {std::num::NonZeroU32};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};
use {worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationFailure, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordStreamFailureKind, worth_store::physical_runtime::RecordWriteSource, worth_store::physical_runtime::RecordWriteSourceError, worth_store::physical_runtime::ServingPhysicalRuntime};

#[test]
fn malformed_stream_contracts_fail_exactly_without_effect_or_idempotency_binding() {
    let parent = tempfile::tempdir().unwrap();
    let media_owner = media(&parent.path().join("store"));
    let policy = durability_with_group_limit(&media_owner, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media_owner.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );

    assert_source_failure(
        &serving,
        placement,
        SourceFailureCase {
            source: crate::physical_record_journeys::durability_admission::stream_failure::MalformedSource::Rejecting,
            material: 51,
            expected: worth_store::physical_runtime::RecordStreamFailureKind::ProducerRejected,
            completed_bytes: 0,
        },
    );
    assert_source_failure(
        &serving,
        placement,
        SourceFailureCase {
            source: crate::physical_record_journeys::durability_admission::stream_failure::MalformedSource::Excess { first: true },
            material: 52,
            expected: worth_store::physical_runtime::RecordStreamFailureKind::SourceExceededDeclaredLength,
            completed_bytes: 2,
        },
    );
    assert_source_failure(
        &serving,
        placement,
        SourceFailureCase {
            source: crate::physical_record_journeys::durability_admission::stream_failure::MalformedSource::InvalidCount,
            material: 53,
            expected: worth_store::physical_runtime::RecordStreamFailureKind::InvalidTransferCount,
            completed_bytes: 0,
        },
    );
    serving.close();
}
