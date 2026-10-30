//! The real recovery composition transfers one pool owner, never a new pool.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::recovery_pool_handoff::*;
use crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family;

#[test]
fn genuine_release_recovery_moves_one_pool_through_core_seal_and_serving() {
    std::thread::Builder::new()
        .name("release-recovery-pool-handoff".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, receipt, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            checkpoint(world.serving(), [0xc1; 32]);
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);

            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine release media must recover: {outcome:?}");
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0);
            let disposed_evidence_bytes = crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::retained_evidence_bytes(&handoff);
            let sample_allocations = handoff.core().certification_residency_allocations();
            let observations = crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::wal_observation_clone::share_without_allocation(&handoff);
            let observation_bytes = observations.charged_bytes();
            assert!(observation_bytes > 0);
            let before_core = sample_allocations.snapshot();
            let core = handoff.into_core();
            let after_core = sample_allocations.snapshot();
            assert_eq!(after_core.pool(), before_core.pool());
            for dimension in [
                worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
                worth_store::physical_runtime::PhysicalResidencyDimension::OperationBytes,
                worth_store::physical_runtime::PhysicalResidencyDimension::TotalBytes,
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
                after_core.for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes),
                before_core.for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes)
            );
            assert!(
                !observations.wal().is_empty(),
                "the clone survives handoff disposal"
            );
            drop(observations);
            let after_observations = sample_allocations.snapshot();
            for dimension in [
                worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
                worth_store::physical_runtime::PhysicalResidencyDimension::OperationBytes,
                worth_store::physical_runtime::PhysicalResidencyDimension::TotalBytes,
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
            let retained_fingerprint_bytes = crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::retained_fingerprint_bytes(&core);
            let checkpoint_facts = checkpoint_observer.facts();
            assert!(retained_checkpoint_bytes > 0);
            let source_reads = core
                .certification_residency_allocations()
                .snapshot()
                .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery));
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
                .certification_begin_recovery_allocation(std::num::NonZeroU64::new(512).unwrap())
                .expect("real Recovery reservation in the carried pool");
            let allocations = core.certification_residency_allocations();
            let initial = allocations.snapshot();
            assert_eq!(held.observation().pool(), initial.pool());
            assert_eq!(
                initial
                    .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery))
                    .active_units(),
                held.bytes() + retained_checkpoint_bytes + retained_fingerprint_bytes
            );
            assert!(
                initial
                    .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes)
                    .active_units()
                    > 0
            );
            let seal = core
                .certification_release_media_into_checkpoint_custody()
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
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal);
            let after_serving = allocations.snapshot();
            // Serving entry funds the standing capture reservation in the
            // same carried pool before Serving is sealed.
            let standing = serving
                .certification_checkpoint_capture_custody_bytes()
                .expect("Serving entry funds the selected capture reservation");
            assert_eq!(
                after_serving
                    .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery))
                    .active_units(),
                held.bytes() + retained_checkpoint_bytes + standing,
                "the escaped checkpoint observer and reservation survive owner transfer"
            );
            let before = initial.for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery));
            let after = after_serving.for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery));
            assert_eq!(before.active_units() + standing - after.active_units(), retained_fingerprint_bytes,
                "successful freshness validation disposes the retained WAL fingerprint");
            assert_eq!(after.released_units() - before.released_units(),
                retained_fingerprint_bytes + after.admitted_units() - before.admitted_units() - standing,
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
                    .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes)
                    .active_units()
                    > 0
            );
            checkpoint(&serving, [0xc2; 32]);
            let before_release = allocations
                .snapshot()
                .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery))
                .active_units();
            let held_bytes = held.bytes();
            drop(held);
            assert_eq!(
                allocations
                    .snapshot()
                    .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery))
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
                    .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery))
                    .active_units(),
                retained_checkpoint_bytes
            );
            assert_eq!(checkpoint_observer.facts(), checkpoint_facts);
            drop(checkpoint_observer);
            crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::assert_disposed("Serving disposal", |dimension| {
                let counters = allocations.snapshot().for_dimension(dimension);
                (counters.active_units(), counters.admitted_units(), counters.released_units())
            });

            let core = recover_core(root);
            let abandoned_core = core.certification_residency_allocations();
            assert_ne!(abandoned_core.snapshot().pool(), initial.pool());
            drop(core);
            crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::assert_disposed("Core abandonment", |dimension| {
                let counters = abandoned_core.snapshot().for_dimension(dimension);
                (counters.active_units(), counters.admitted_units(), counters.released_units())
            });

            let core = recover_core(root);
            let abandoned_seal = core.certification_residency_allocations();
            let seal = core
                .certification_release_media_into_checkpoint_custody()
                .expect("fresh recovered seal");
            drop(seal);
            crate::certified_release_serving::release_capacity_budget::recovery_pool_handoff::backing_census::assert_disposed("seal abandonment", |dimension| {
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
            let (world, receipt, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
            assert!(receipt.remaining_payload_records() > 0);
            checkpoint(world.serving(), [0xc3; 32]);
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let before = snapshot_family(root);
            let core = recover_core(root);
            let allocations = core.certification_residency_allocations();
            let seal = core.certification_release_media_into_checkpoint_custody().expect("genuine current seal");
            let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap());
            let policy = policy_with_different_recovery_scope(format);
            let outcome = open_with_policy(root, seal, format, policy).into_raw();
            let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
                panic!("changed policy must deny at Serving admission")
            };
            assert_eq!(denial.reason(), worth_store::physical_runtime::RecordBootstrapDenial::RecoveredResidencyPolicyMismatch);
            assert_eq!(snapshot_family(root), before);
            for dimension in [worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes, worth_store::physical_runtime::PhysicalResidencyDimension::OperationBytes, worth_store::physical_runtime::PhysicalResidencyDimension::TotalBytes] {
                let counters = allocations.snapshot().for_dimension(dimension);
                assert_eq!(counters.active_units(), 0, "policy denial: {dimension:?}");
                assert_eq!(counters.admitted_units(), counters.released_units());
            }
        })
        .expect("policy drift worker")
        .join()
        .expect("policy drift worker did not panic");
}
#[path = "serving_capture_reservation/mod.rs"]
pub(crate) mod serving_capture_reservation;
#[path = "serving_effect_freshness/mod.rs"]
pub(crate) mod serving_effect_freshness;
#[path = "serving_head_freshness/mod.rs"]
pub(crate) mod serving_head_freshness;
#[path = "serving_wal_pressure/mod.rs"]
pub(crate) mod serving_wal_pressure;
