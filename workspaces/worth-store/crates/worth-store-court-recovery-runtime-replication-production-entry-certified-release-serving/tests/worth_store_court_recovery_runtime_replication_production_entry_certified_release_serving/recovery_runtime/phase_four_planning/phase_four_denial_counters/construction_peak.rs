use crate::phase_four_planning::ordinary_recovery_declaration;
use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointRequest, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationSuccess,
    PhysicalMutationRequest, PhysicalRecordFormatDeclaration, PhysicalWalGroupAppendOutcome,
    PhysicalWalGroupBarrierOutcome, RecordAppendBatch,
};
use worth_store_recovery_physics::RecoveryPlanCostDenial;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits, PhysicalRecoveryPlanningDenial,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};
use worth_store_testkit_recovery_runtime::phase_three_support::{
    admitted_recovery_with_limits, expect_blocked,
};

#[test]
fn later_staging_refusal_preserves_the_larger_construction_peak_without_effects() {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_operation_budget(
        "planning-construction-refusal-peak",
        format,
        64,
        std::num::NonZeroU64::new(128 * 1024 * 1024).unwrap(),
    )
    .unwrap();
    let root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x71; 32], b"checkpoint-source");
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x72; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("source checkpoint must start");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    // Stay within the existing 16 MiB WAL segment while retaining enough real
    // frame identities for staging scratch to exceed capture's common peak.
    append_pending_extent(&world, &vec![0x5a; 14 * 1024 * 1024]);
    drop(world);

    let mut early = ordinary_recovery_declaration(4_096);
    early.staging_bytes = 3_276_799;
    early.wal_bytes = 32 * 1024 * 1024;
    early.redo_bytes = 32 * 1024 * 1024;
    early.recovery_memory_bytes = 128 * 1024 * 1024;
    early.observation_bytes = 128 * 1024 * 1024;
    let mut later = ordinary_recovery_declaration(4_096);
    later.dirty_frames = 1;
    later.wal_bytes = early.wal_bytes;
    later.redo_bytes = early.redo_bytes;
    later.recovery_memory_bytes = early.recovery_memory_bytes;
    later.observation_bytes = early.observation_bytes;
    let mut peaks = Vec::new();
    for (declaration, expected, dimension) in [
        (
            early,
            RecoveryPlanCostDenial::StagingBytes,
            PhysicalRecoveryLimitDimension::StagingBytes,
        ),
        (
            later,
            RecoveryPlanCostDenial::DirtyFrames,
            PhysicalRecoveryLimitDimension::DirtyFrames,
        ),
    ] {
        let selected = admitted_recovery_with_limits(
            root.path(),
            PhysicalRecoveryLimits::admit(declaration).unwrap(),
        )
        .discover()
        .unwrap()
        .select()
        .unwrap();
        let blocked = match selected.plan() {
            Ok(_) => panic!("the original staging limit must refuse before execution"),
            Err(outcome) => expect_blocked(outcome),
        };
        assert_eq!(
            blocked.evidence().planning_denial,
            Some(PhysicalRecoveryPlanningDenial::Cost(expected)),
            "unexpected refusal: {blocked:?}"
        );
        assert_eq!(blocked.cause().limit().unwrap().dimension(), dimension);
        assert_eq!(blocked.recovery_effects(), 0);
        peaks.push(
            blocked
                .evidence()
                .planning_counters
                .unwrap()
                .peak_recovery_bytes(),
        );
    }
    assert!(
        peaks[1] > peaks[0],
        "later scratch must remain measured: {peaks:?}"
    );
}

fn append_pending_extent(world: &PhysicalResidencyStoreWorld, bytes: &[u8]) {
    let submission = world.serving().certification_record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x73; 32]))
        .unwrap();
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([bytes]).unwrap(),
                world.placement(),
                request,
            )
            .into_raw()
    else {
        panic!("real pending extent must prepare");
    };
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("real pending extent must append"),
    };
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
}
