#[allow(unused_imports)]
use crate::retirement_obligation::*;
use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRetirementDenial,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::RecoveryCleanupDispositionKind,
    worth_store_recovery_runtime::RecoveryCleanupTarget,
    worth_store_recovery_runtime::WorthStoreRecovery,
};
use {
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
use {
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits,
};

#[test]
fn recovery_reconstructs_an_unresolved_retirement_intent() {
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c10-retirement-obligation").unwrap();
    let retained = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x61; 32], b"retirement-obligation");
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x62; 32]),
        )
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_segment(
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(1_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("rewrite preparation must return a mutation")
        }
        worth_proof::TransitionOutcome::Denied(_) => panic!("rewrite preparation was denied"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("rewrite preparation was deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("rewrite preparation was stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("rewrite preparation required rebind")
        }
        worth_proof::TransitionOutcome::Failed(_) => panic!("rewrite preparation failed"),
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite must publish: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite must settle: {:?}", fate.stage())
        }
    }
    world
        .serving()
        .certification_stop_before_retirement_delete();
    assert_eq!(
        world.serving().retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Delete)
    );
    world.close();
    let root = retained.persist();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        recovery_request_with_limits(&root, ordinary_limits()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
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
        .filter(|disposition| matches!(
            disposition.target(),
            worth_store_recovery_runtime::RecoveryCleanupTarget::Wal(_)
        ))
        .all(|disposition| disposition.kind()
            != worth_store_recovery_runtime::RecoveryCleanupDispositionKind::Eligible));
}
