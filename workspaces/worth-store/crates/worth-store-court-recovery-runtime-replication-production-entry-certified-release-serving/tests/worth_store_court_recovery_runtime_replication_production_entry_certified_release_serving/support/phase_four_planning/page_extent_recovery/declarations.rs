pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_recovery_physics::PhysicalRedoTargetIdentity, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::canonical_rooted_mutation_without_acknowledgment, worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked, crate::phase_four_planning::selected_ordinary_recovery};
pub(crate) use {std::io::Read, std::io::Seek, std::io::SeekFrom, std::io::Write};
pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome};

pub(crate) fn prepare_extent_recovery_root(name: &str) -> worth_store_test_support::TemporaryDirectory {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(name).unwrap();
    let retained_root = world.retained_root();
    let payload = vec![0x61; 40_000];
    canonical_physical_mutation_acknowledgment(&world, [0x51; 32], &payload);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x52; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("extent checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_rooted_mutation_without_acknowledgment(&world, [0x53; 32], &payload);
    canonical_durable_wal_attempt_without_execution(&world, [0x54; 32], &payload);
    drop(world);
    retained_root
}