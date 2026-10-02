//! The real recovery composition transfers one pool owner, never a new pool.

use super::*;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
    RecoveredPhysicalRuntimeCore,
};

#[path = "recovery_pool_handoff/backing_census.rs"]
mod backing_census;
#[path = "recovery_pool_handoff/serving_effect_freshness.rs"]
mod serving_effect_freshness;
#[path = "recovery_pool_handoff/serving_head_freshness.rs"]
mod serving_head_freshness;
#[path = "recovery_pool_handoff/serving_wal_pressure.rs"]
mod serving_wal_pressure;
#[path = "recovery_pool_handoff/wal_observation_clone.rs"]
mod wal_observation_clone;

#[test]
fn genuine_release_recovery_moves_one_pool_through_core_seal_and_serving() {
    std::thread::Builder::new()
        .name("release-recovery-pool-handoff".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            checkpoint(world.serving(), [0xc1; 32]);
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);

            let outcome = WorthStoreRecovery::recover(super::super::request(root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine release media must recover: {outcome:?}");
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0);
            let disposed_evidence_bytes = backing_census::retained_evidence_bytes(&handoff);
            let sample_allocations = handoff.core().certification_residency_allocations();
            let observations = wal_observation_clone::share_without_allocation(&handoff);
            let observation_bytes = observations.charged_bytes();
            assert!(observation_bytes > 0);
            let before_core = sample_allocations.snapshot();
            let core = handoff.into_core();
            let after_core = sample_allocations.snapshot();
            assert_eq!(after_core.pool(), before_core.pool());
            for dimension in [
                Dimension::OperationScope(Scope::Recovery),
                Dimension::OperationBytes,
                Dimension::TotalBytes,
            ] {
                let before = before_core.for_dimension(dimension);
                let after = after_core.for_dimension(dimension);
                assert_eq!(
                    before.active_units() - after.active_units(),
                    disposed_evidence_bytes,
                    "into_core disposes sample and WAL selection before releasing their charges"
                );
                assert_eq!(after.admitted_units(), before.admitted_units());
                assert_eq!(
                    after.released_units() - before.released_units(),
                    disposed_evidence_bytes
                );
            }
            assert_eq!(
                after_core.for_dimension(Dimension::MetadataBytes),
                before_core.for_dimension(Dimension::MetadataBytes)
            );
            assert!(
                !observations.wal().is_empty(),
                "the clone survives handoff disposal"
            );
            drop(observations);
            let after_observations = sample_allocations.snapshot();
            for dimension in [
                Dimension::OperationScope(Scope::Recovery),
                Dimension::OperationBytes,
                Dimension::TotalBytes,
            ] {
                assert_eq!(
                    after_core.for_dimension(dimension).active_units()
                        - after_observations.for_dimension(dimension).active_units(),
                    observation_bytes
                );
                assert_eq!(
                    after_observations.for_dimension(dimension).admitted_units(),
                    after_core.for_dimension(dimension).admitted_units()
                );
            }
            let checkpoint_observer = core.checkpoint().expect("real selected checkpoint").clone();
            let retained_checkpoint_bytes = checkpoint_observer.owned_heap_bytes().unwrap();
            let retained_fingerprint_bytes = backing_census::retained_fingerprint_bytes(&core);
            let checkpoint_facts = checkpoint_observer.facts();
            assert!(retained_checkpoint_bytes > 0);
            let source_reads = core
                .certification_residency_allocations()
                .snapshot()
                .for_dimension(Dimension::OperationScope(Scope::Recovery));
            assert!(
                source_reads.admissions() > 0,
                "production source reads must acquire native backing before allocation"
            );
            assert_eq!(
                source_reads.active_units(),
                retained_checkpoint_bytes + retained_fingerprint_bytes,
                "temporary reads end; checkpoint, WAL and full-tree head witnesses retain native charges"
            );
            assert_eq!(
                source_reads.admitted_units() - source_reads.released_units(),
                retained_checkpoint_bytes + retained_fingerprint_bytes
            );
            let held = core
                .certification_begin_recovery_allocation(NonZeroU64::new(512).unwrap())
                .expect("real Recovery reservation in the carried pool");
            let allocations = core.certification_residency_allocations();
            let initial = allocations.snapshot();
            assert_eq!(held.observation().pool(), initial.pool());
            assert_eq!(
                initial
                    .for_dimension(Dimension::OperationScope(Scope::Recovery))
                    .active_units(),
                held.bytes() + retained_checkpoint_bytes + retained_fingerprint_bytes
            );
            assert!(
                initial
                    .for_dimension(Dimension::MetadataBytes)
                    .active_units()
                    > 0
            );
            let seal = core
                .into_checkpoint_custody()
                .expect("actual rejoined release seal");
            assert_eq!(seal.checkpoint().unwrap().facts(), checkpoint_facts);
            assert_eq!(
                seal.certification_residency_allocations().snapshot().pool(),
                initial.pool()
            );
            assert_eq!(
                allocations.snapshot(),
                initial,
                "seal transfer neither releases nor reopens the pool"
            );
            let serving = super::super::admit_serving_with_seal(root, seal);
            let after_serving = allocations.snapshot();
            assert_eq!(
                after_serving
                    .for_dimension(Dimension::OperationScope(Scope::Recovery))
                    .active_units(),
                held.bytes() + retained_checkpoint_bytes,
                "the escaped checkpoint observer and reservation survive owner transfer"
            );
            let before = initial.for_dimension(Dimension::OperationScope(Scope::Recovery));
            let after = after_serving.for_dimension(Dimension::OperationScope(Scope::Recovery));
            assert_eq!(before.active_units() - after.active_units(), retained_fingerprint_bytes,
                "successful freshness validation disposes the retained WAL fingerprint");
            assert_eq!(after.released_units() - before.released_units(),
                retained_fingerprint_bytes + after.admitted_units() - before.admitted_units(),
                "temporary Serving reads balance independently of fingerprint disposal");
            assert_eq!(
                serving
                    .residency_observation()
                    .allocations()
                    .pool_incarnation(),
                initial.pool().get()
            );
            assert_eq!(
                serving.residency_observation().store_identity(),
                initial.store()
            );
            assert!(
                allocations
                    .snapshot()
                    .for_dimension(Dimension::MetadataBytes)
                    .active_units()
                    > 0
            );
            checkpoint(&serving, [0xc2; 32]);
            let before_release = allocations
                .snapshot()
                .for_dimension(Dimension::OperationScope(Scope::Recovery))
                .active_units();
            let held_bytes = held.bytes();
            drop(held);
            assert_eq!(
                allocations
                    .snapshot()
                    .for_dimension(Dimension::OperationScope(Scope::Recovery))
                    .active_units(),
                before_release - held_bytes
            );
            let closed = serving.close();
            assert!(
                closed.residency().requires_inspection(),
                "live escaped backing must be reported, not silently uncharged"
            );
            assert_eq!(
                allocations
                    .snapshot()
                    .for_dimension(Dimension::OperationScope(Scope::Recovery))
                    .active_units(),
                retained_checkpoint_bytes
            );
            assert_eq!(checkpoint_observer.facts(), checkpoint_facts);
            drop(checkpoint_observer);
            backing_census::assert_disposed("Serving disposal", |dimension| {
                let counters = allocations.snapshot().for_dimension(dimension);
                (counters.active_units(), counters.admitted_units(), counters.released_units())
            });

            let core = recover_core(root);
            let abandoned_core = core.certification_residency_allocations();
            assert_ne!(abandoned_core.snapshot().pool(), initial.pool());
            drop(core);
            backing_census::assert_disposed("Core abandonment", |dimension| {
                let counters = abandoned_core.snapshot().for_dimension(dimension);
                (counters.active_units(), counters.admitted_units(), counters.released_units())
            });

            let core = recover_core(root);
            let abandoned_seal = core.certification_residency_allocations();
            let seal = core
                .into_checkpoint_custody()
                .expect("fresh recovered seal");
            drop(seal);
            backing_census::assert_disposed("seal abandonment", |dimension| {
                let counters = abandoned_seal.snapshot().for_dimension(dimension);
                (counters.active_units(), counters.admitted_units(), counters.released_units())
            });
        })
        .expect("recovery pool worker")
        .join()
        .expect("recovery pool worker did not panic");
}

#[test]
fn changed_serving_policy_denies_without_reopening_pool_or_changing_media() {
    std::thread::Builder::new()
        .name("release-recovery-policy-drift".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            checkpoint(world.serving(), [0xc3; 32]);
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let before = snapshot_family(root);
            let core = recover_core(root);
            let allocations = core.certification_residency_allocations();
            let seal = core.into_checkpoint_custody().expect("genuine current seal");
            let format = AdmittedPhysicalRecordFormat::admit(PhysicalRecordFormatDeclaration::builder().admit().unwrap());
            let policy = policy_with_different_recovery_scope(format);
            let outcome = open_with_policy(root, seal, format, policy).into_raw();
            let TransitionOutcome::Denied(denial) = outcome else {
                panic!("changed policy must deny at Serving admission")
            };
            assert_eq!(denial.reason(), worth_store::physical_runtime::RecordBootstrapDenial::RecoveredResidencyPolicyMismatch);
            assert_eq!(snapshot_family(root), before);
            for dimension in [Dimension::MetadataBytes, Dimension::OperationBytes, Dimension::TotalBytes] {
                let counters = allocations.snapshot().for_dimension(dimension);
                assert_eq!(counters.active_units(), 0, "policy denial: {dimension:?}");
                assert_eq!(counters.admitted_units(), counters.released_units());
            }
        })
        .expect("policy drift worker")
        .join()
        .expect("policy drift worker did not panic");
}

fn recover_core(root: &Path) -> RecoveredPhysicalRuntimeCore {
    let outcome = WorthStoreRecovery::recover(super::super::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("genuine release media must recover: {outcome:?}")
    };
    assert_eq!(handoff.core().recovery_effect_count(), 0);
    handoff.into_core()
}

fn checkpoint(serving: &worth_store::physical_runtime::ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("genuine release checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "release checkpoint: {outcome:?}"
    );
}

fn policy_with_different_recovery_scope(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use worth_store::physical_runtime::{
        PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind as Kind,
    };
    let canonical = AdmittedPhysicalRecordResidencyPolicy::canonical(format);
    let bytes = |value| NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(canonical.total_bytes()))
        .resident_bytes(bytes(canonical.resident_bytes()))
        .metadata_bytes(bytes(canonical.metadata_bytes()))
        .frame_entries(count(canonical.frame_entries()))
        .pinned_frames(count(canonical.pinned_frames()))
        .pin_leases(count(canonical.pin_leases()))
        .dirty_frames(count(canonical.dirty_frames()))
        .dirty_replacement_bytes(bytes(canonical.dirty_replacement_bytes()))
        .operation_bytes(bytes(canonical.operation_bytes()))
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
        let value = canonical.scope_bytes(scope) - u64::from(scope == Scope::Recovery);
        builder = builder.scope_bytes(scope, bytes(value));
    }
    for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
        builder = builder.speculative_frames(kind, count(canonical.speculative_frames(kind)));
    }
    builder
        .admit(format)
        .into_result()
        .expect("only Recovery scope differs")
}

fn open_with_policy(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    format: AdmittedPhysicalRecordFormat,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) -> worth_store::physical_runtime::RecordStoreOpenOutcome {
    use std::num::NonZeroU32;
    use worth_store::physical_runtime::*;
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("real Serving media must admit")
    };
    let TransitionOutcome::Success(durability) = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::MIN),
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
        panic!("real Serving durability must admit")
    };
    media.open_record_store(
        PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(policy)
            .with_recovered_checkpoint_custody(seal),
    )
}
