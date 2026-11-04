use crate::phase_four_planning::ordinary_recovery_declaration;
use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointRequest, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationSuccess,
    PhysicalMutationRequest, PhysicalOperationAllocationScope, PhysicalRecordFormatDeclaration,
    PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind, PhysicalWalGroupAppendOutcome,
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
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery_with_residency_and_wal_budgets(
            "planning-construction-refusal-peak",
            format,
            64,
            residency(format),
            std::num::NonZeroU64::new(32 * 1024 * 1024).unwrap(),
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
    // Two 7 KiB records fit in each admitted 16 KiB page; 1,792 records
    // create 896 inline frame identities instead of one arena frame.
    let record = vec![0x5a; 7 * 1024];
    let batch =
        RecordAppendBatch::try_from_iter(std::iter::repeat_n(record.as_slice(), 1_792)).unwrap();
    append_pending_pages(&world, batch);
    drop(world);

    let mut early = ordinary_recovery_declaration(4_096);
    early.staging_bytes = 3_276_799;
    early.wal_bytes = 32 * 1024 * 1024;
    early.redo_bytes = 32 * 1024 * 1024;
    early.recovery_memory_bytes = 256 * 1024 * 1024;
    early.observation_bytes = 256 * 1024 * 1024;
    let mut later = ordinary_recovery_declaration(4_096);
    later.dirty_frames = 1;
    later.wal_bytes = early.wal_bytes;
    later.redo_bytes = early.redo_bytes;
    later.recovery_memory_bytes = early.recovery_memory_bytes;
    later.observation_bytes = early.observation_bytes;
    let (early_peak, root_bytes) = refusal(
        root.path(),
        early,
        RecoveryPlanCostDenial::StagingBytes,
        PhysicalRecoveryLimitDimension::StagingBytes,
    );
    // Fund the actual owner's root requirement plus exactly the first page.
    // The second distinct frame must reach DirtyFrames before another byte charge.
    later.staging_bytes = root_bytes
        .checked_add(u64::from(format.declaration().page_size().bytes()))
        .unwrap();
    let (later_peak, _) = refusal(
        root.path(),
        later,
        RecoveryPlanCostDenial::DirtyFrames,
        PhysicalRecoveryLimitDimension::DirtyFrames,
    );
    assert!(
        later_peak > early_peak,
        "later scratch must remain measured: {early_peak}, {later_peak}"
    );
}

fn refusal(
    root: &std::path::Path,
    declaration: worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    expected: RecoveryPlanCostDenial,
    dimension: PhysicalRecoveryLimitDimension,
) -> (u64, u64) {
    let selected =
        admitted_recovery_with_limits(root, PhysicalRecoveryLimits::admit(declaration).unwrap())
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
    let limit = blocked.cause().limit().unwrap();
    assert_eq!(limit.dimension(), dimension);
    assert_eq!(blocked.recovery_effects(), 0);
    (
        blocked
            .evidence()
            .planning_counters
            .unwrap()
            .peak_recovery_bytes(),
        limit.observed(),
    )
}

fn append_pending_pages(world: &PhysicalResidencyStoreWorld, batch: RecordAppendBatch) {
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
            .prepare_durable_append(batch, world.placement(), request)
            .into_raw()
    else {
        panic!("real pending pages must prepare");
    };
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        PhysicalWalGroupAppendOutcome::NotAdmitted { cause, .. } => {
            panic!("pending pages append not admitted: {cause:?}")
        }
        PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => {
            panic!("pending pages group rejected: {:?}", rejected.cause())
        }
        PhysicalWalGroupAppendOutcome::NotStarted(continuation) => panic!(
            "pending pages append not started: {:?}",
            continuation.cause()
        ),
        PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation) => {
            panic!("pending pages append partial: {:?}", continuation.cause())
        }
        PhysicalWalGroupAppendOutcome::Indeterminate(_) => {
            panic!("pending pages append indeterminate")
        }
    };
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
}

fn residency(
    format: AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    let bytes = |count| std::num::NonZeroU64::new(count).unwrap();
    let frames = |count| std::num::NonZeroU32::new(count).unwrap();
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(336 * 1024 * 1024))
        .resident_bytes(bytes(32 * 1024 * 1024))
        .dirty_replacement_bytes(bytes(32 * 1024 * 1024))
        .metadata_bytes(bytes(16 * 1024 * 1024))
        .operation_bytes(bytes(256 * 1024 * 1024))
        .frame_entries(frames(2_048))
        .pinned_frames(frames(2_048))
        .pin_leases(frames(2_048))
        .dirty_frames(frames(2_048));
    for scope in [
        PhysicalOperationAllocationScope::ForegroundRead,
        PhysicalOperationAllocationScope::ForegroundWrite,
        PhysicalOperationAllocationScope::Recovery,
        PhysicalOperationAllocationScope::Scrub,
        PhysicalOperationAllocationScope::Maintenance,
        PhysicalOperationAllocationScope::Verification,
        PhysicalOperationAllocationScope::Blob,
    ] {
        builder = builder.scope_bytes(scope, bytes(256 * 1024 * 1024));
    }
    for kind in [
        PhysicalSpeculativeWorkKind::Prefetch,
        PhysicalSpeculativeWorkKind::ReadAhead,
        PhysicalSpeculativeWorkKind::WriteBehind,
    ] {
        builder = builder.speculative_frames(kind, frames(8));
    }
    builder.admit(format).into_result().unwrap()
}
