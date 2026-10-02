//! Recovery carries one pool into Serving while retaining its original ceiling.
//! The larger or smaller target policy is declared before recovery admission.

use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, CheckpointMemoryLimit, FilesystemAccessPosture,
    FilesystemMediaAdmission, GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy,
    PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy,
    PhysicalOperationAllocationScope as Scope, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen, PhysicalRecordResidencyPolicy,
    PhysicalResidencyDimension, PhysicalRuntimeAdmission,
    PhysicalSpeculativeWorkKind as Speculation, PhysicalStore, PhysicalWalPolicy,
    RecoveredPhysicalCheckpointCustody, RetainedWalTailLimit, ServingPhysicalRuntime,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

use super::*;

const ORIGINAL_RECOVERY_BYTES: u64 = 24 << 20;
const LOWER_TARGET_BYTES: u64 = 16 << 20;

#[test]
fn original_recovery_ceiling_survives_seal_checkpoint_and_fresh_reopen() {
    let (world, receipt, _) = release_reopen::released_world(1024);
    assert_eq!(receipt.remaining_payload_records(), 0);
    let store = world.serving().residency_observation().store_identity();
    checkpoint(world.serving(), [0xb1; 32]);
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("original-recovery-admission-serving".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let first = recover_seal(&root, store, PhysicalRecoveryStaticConfiguration::current());
            let serving = super::admit_serving_with_seal(&root, first);
            let observation = serving.residency_observation();
            let target = observation.admitted_policy().scope_bytes(Scope::Recovery);
            assert!(target > ORIGINAL_RECOVERY_BYTES);
            assert_eq!(observation.store_identity(), store);
            let counters = observation.counters();
            assert_eq!(counters.peak_operation_bytes(), ORIGINAL_RECOVERY_BYTES);
            assert_eq!(counters.active_operation_bytes(), 0);
            let grants = observation
                .allocations()
                .for_dimension(PhysicalResidencyDimension::OperationBytes);
            assert!(grants.admissions() > 0);
            assert!(grants.admitted_units() >= ORIGINAL_RECOVERY_BYTES);
            // The original ceiling must constrain the real Serving issuer,
            // not just a recovery sizing hint. The larger target policy would
            // admit this competing request without the carried native clamp.
            let issuer = serving.physical_allocations();
            let held = issuer
                .admit_recovery(NonZeroU64::MIN)
                .expect("one byte fits the genuine recovered pool");
            let before = serving.residency_observation().allocations();
            let denial =
                match issuer.admit_recovery(NonZeroU64::new(ORIGINAL_RECOVERY_BYTES).unwrap()) {
                    Err(denial) => denial,
                    Ok(_) => panic!("Serving cannot spend beyond the original aggregate ceiling"),
                };
            let pressure = denial
                .pressure()
                .expect("original native Recovery pressure");
            assert_eq!(pressure.scope(), Scope::Recovery);
            assert_eq!(
                pressure.dimension(),
                PhysicalResidencyDimension::OperationScope(Scope::Recovery),
            );
            assert_eq!(pressure.requested(), ORIGINAL_RECOVERY_BYTES);
            assert_eq!(pressure.admitted(), 1);
            assert_eq!(pressure.limit(), ORIGINAL_RECOVERY_BYTES);
            assert!(!pressure.effect_may_have_started());
            let after = serving.residency_observation().allocations();
            let dimension = PhysicalResidencyDimension::OperationScope(Scope::Recovery);
            assert_eq!(after.for_dimension(dimension).active_units(), 1);
            assert_eq!(
                after.for_dimension(dimension).admitted_units(),
                before.for_dimension(dimension).admitted_units(),
            );
            drop(held);
            checkpoint(&serving, [0xb2; 32]);
            assert_eq!(
                serving
                    .residency_observation()
                    .counters()
                    .peak_operation_bytes(),
                ORIGINAL_RECOVERY_BYTES,
                "checkpoint work must not reopen the larger target operation grant"
            );
            assert!(!serving.close().residency().requires_inspection());

            // Admit the smaller target pool before fresh C8 recovery. Its
            // scope and the original recovery ceiling both continue to hold.
            let format = AdmittedPhysicalRecordFormat::admit(
                PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            );
            let second = recover_seal(
                &root,
                store,
                PhysicalRecoveryStaticConfiguration::current()
                    .with_residency_policy(lower_target_policy(format))
                    .expect("lower target policy admits the configured format"),
            );
            let serving = open_with_lower_target_policy(&root, second);
            let observation = serving.residency_observation();
            assert_eq!(observation.store_identity(), store);
            assert_eq!(
                observation.admitted_policy().scope_bytes(Scope::Recovery),
                LOWER_TARGET_BYTES
            );
            let peak = observation.counters().peak_operation_bytes();
            assert!(peak > 0 && peak < ORIGINAL_RECOVERY_BYTES);
            assert!(peak <= LOWER_TARGET_BYTES);
            checkpoint(&serving, [0xb3; 32]);
            assert!(
                serving
                    .residency_observation()
                    .counters()
                    .peak_operation_bytes()
                    <= LOWER_TARGET_BYTES
            );
            assert!(!serving.close().residency().requires_inspection());
        })
        .expect("recovery worker");
    worker.join().expect("recovery worker did not panic");
}

fn recover_seal(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    configuration: PhysicalRecoveryStaticConfiguration,
) -> RecoveredPhysicalCheckpointCustody {
    let outcome = WorthStoreRecovery::recover(super::recovery_request::request_with_configuration(
        root,
        ORIGINAL_RECOVERY_BYTES,
        configuration,
    ));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("genuine selected release recovery denied: {outcome:?}")
    };
    let core = handoff.into_core();
    let admission = core.recovery_allocation_admission();
    assert_eq!(admission.store_identity(), store);
    assert_eq!(admission.byte_limit(), ORIGINAL_RECOVERY_BYTES);
    let seal = core
        .into_checkpoint_custody()
        .expect("selected release grants a one-shot custody seal");
    assert_eq!(seal.recovery_allocation_admission(), admission);
    seal
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("genuine released checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "genuine released checkpoint: {outcome:?}"
    );
}

fn open_with_lower_target_policy(
    root: &Path,
    seal: RecoveredPhysicalCheckpointCustody,
) -> ServingPhysicalRuntime {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("Serving media must admit")
    };
    let TransitionOutcome::Success(durability) = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 << 20).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    else {
        panic!("Serving durability must admit")
    };
    let open = PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(lower_target_policy(format))
        .with_recovered_checkpoint_custody(seal);
    let TransitionOutcome::Success(serving) = media.open_record_store(open).into_raw() else {
        panic!("smaller target policy must open from genuine custody seal")
    };
    serving
}

fn lower_target_policy(
    format: AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    let bytes = |value| NonZeroU64::new(value).unwrap();
    let count = |value| NonZeroU32::new(value).unwrap();
    let mut policy = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(384 << 20))
        .resident_bytes(bytes(64 << 20))
        .metadata_bytes(bytes(3 << 20))
        .frame_entries(count(4096))
        .pinned_frames(count(256))
        .pin_leases(count(512))
        .dirty_frames(count(64))
        .dirty_replacement_bytes(bytes(64 << 20))
        .operation_bytes(bytes(LOWER_TARGET_BYTES))
        .progress_headroom_bytes(64 << 10);
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        policy = policy.scope_bytes(scope, bytes(LOWER_TARGET_BYTES));
    }
    policy
        .speculative_frames(Speculation::Prefetch, count(256))
        .speculative_frames(Speculation::ReadAhead, count(256))
        .speculative_frames(Speculation::WriteBehind, count(64))
        .admit(format)
        .into_result()
        .expect("finite lower target policy must admit")
}
