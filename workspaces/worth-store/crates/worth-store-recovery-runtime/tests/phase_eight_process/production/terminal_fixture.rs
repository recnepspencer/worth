use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    canonical_rooted_mutation_without_acknowledgment, PhysicalResidencyStoreWorld,
};

/// Build a retained root only for the certification-only terminal profile
/// tests. Process-death evidence uses the shipped C8 writer in `harness`.
pub fn certification_persisted_root(label: &str) -> worth_store_test_support::TemporaryDirectory {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x91; 32], b"c8-terminal-acknowledged");
    complete_checkpoint(&world);
    canonical_rooted_mutation_without_acknowledgment(&world, [0x92; 32], b"c8-terminal-rooted");
    canonical_durable_wal_attempt_without_execution(&world, [0x93; 32], b"c8-terminal-wal");
    drop(world);
    retained_root
}

fn complete_checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x94; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("terminal fixture checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}
