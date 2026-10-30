pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::canonical_rooted_mutation_without_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

/// Build a retained root only for the certification-only terminal profile
/// tests. Process-death evidence uses the shipped C8 writer in `harness`.
pub(crate) fn certification_persisted_root(
    label: &str,
) -> worth_store_test_support::TemporaryDirectory {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x91; 32], b"c8-terminal-acknowledged");
    complete_checkpoint(&world);
    canonical_rooted_mutation_without_acknowledgment(&world, [0x92; 32], b"c8-terminal-rooted");
    canonical_durable_wal_attempt_without_execution(&world, [0x93; 32], b"c8-terminal-wal");
    drop(world);
    retained_root
}

pub(crate) fn complete_checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x94; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("terminal fixture checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}
