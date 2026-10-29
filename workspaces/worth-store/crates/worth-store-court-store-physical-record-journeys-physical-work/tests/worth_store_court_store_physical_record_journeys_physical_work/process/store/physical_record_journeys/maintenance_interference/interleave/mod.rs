#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::maintenance_interference::interleave::*;
use {std::sync::atomic::AtomicBool, std::sync::atomic::Ordering};
use {std::thread};
use {std::time::Duration, std::time::Instant};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest, worth_store::physical_runtime::PhysicalIntegrityScrubTarget, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordReadLimits};
use {worth_store_physical_backend::MediaOperationRole};
use {worth_store_physical_format::PhysicalArtifactReadTarget, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RecordArtifactFile};
use {worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange};

use {worth_store::physical_runtime::PhysicalWorkCapacity};

use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint, worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement, worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files};

#[test]
fn cold_extent_survives_four_publications_scrub_and_blocked_retirement() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (profile, request, _) = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(8, 256, 32_768, 1024 * 1024, 64 * 1024 * 1024)
        .unwrap()
        .with_dispatch_permits(4)
        .unwrap();
    let (serving, gate, activation) =
        worth_store_testkit_store::physical_record_journeys::maintenance_interference::sync::open_with_file_sync_pause(&root, profile.with_capacity(capacity));
    run_cold_extent(&serving, gate, activation, &root, request, 0);
    serving.close();
}
