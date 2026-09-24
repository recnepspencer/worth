#[allow(dead_code)]
mod phase_three_support;

use phase_three_support::{limit_declaration, recovery_request_with_limits};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRetirementDenial,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOutcome, RecoveryCleanupDispositionKind,
    RecoveryCleanupTarget, WorthStoreRecovery,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

fn ordinary_limits() -> PhysicalRecoveryLimits {
    let mut declaration = limit_declaration(2, 8, 2 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
    declaration.wal_bytes = 2 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 4 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 32 * 1024 * 1024;
    declaration.recovery_memory_bytes = 32 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.observation_bytes = 32 * 1024 * 1024;
    PhysicalRecoveryLimits::admit(declaration).unwrap()
}

#[test]
fn recovery_reconstructs_an_unresolved_retirement_intent() {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("c10-retirement-obligation").unwrap();
    let retained = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x61; 32], b"retirement-obligation");
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x62; 32]))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_segment(
            world.placement(),
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("rewrite preparation must return a mutation"),
        TransitionOutcome::Denied(_) => panic!("rewrite preparation was denied"),
        TransitionOutcome::Deferred(_) => panic!("rewrite preparation was deferred"),
        TransitionOutcome::Stale(_) => panic!("rewrite preparation was stale"),
        TransitionOutcome::RebindRequired(_) => panic!("rewrite preparation required rebind"),
        TransitionOutcome::Failed(_) => panic!("rewrite preparation failed"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite must publish: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite must settle: {:?}", fate.stage())
        }
    }
    world
        .serving()
        .certification_stop_before_retirement_delete();
    assert_eq!(
        world.serving().retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Delete)
    );
    world.close();
    let root = retained.persist();
    let outcome =
        WorthStoreRecovery::recover(recovery_request_with_limits(&root, ordinary_limits()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("an unresolved retirement intent must not reject recovery: {outcome:?}")
    };
    let retirements = handoff.freshness_sample().retirements();
    assert_eq!(retirements.len(), 1);
    assert!(retirements[0].bytes() > 0);
    assert!(handoff
        .cleanup_posture()
        .evidence()
        .dispositions()
        .iter()
        .filter(|disposition| matches!(disposition.target(), RecoveryCleanupTarget::Wal(_)))
        .all(|disposition| disposition.kind() != RecoveryCleanupDispositionKind::Eligible));
}
