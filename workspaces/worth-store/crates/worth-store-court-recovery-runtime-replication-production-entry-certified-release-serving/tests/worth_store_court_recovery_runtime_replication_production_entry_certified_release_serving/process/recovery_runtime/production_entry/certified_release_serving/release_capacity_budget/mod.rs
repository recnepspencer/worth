//! The first V3 release reserves its complete recovery closure before any media effect.

#[allow(unused_imports)]
use crate::production_entry::certified_release_serving::release_capacity_budget::*;
use {std::fs, std::num::NonZeroU64, std::path::PathBuf};
use {
    std::num::NonZeroU16, std::path::Path, worth_proof::AdmittedBlobReleaseProof,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run_world,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};
use {
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial,
};
use {
    worth_store_physical_format::ReleaseCustodyHeadEntryV1,
    worth_store_physical_format::ReleaseCustodyHeadKeyV1,
    worth_store_physical_format::BLOB_CONTROL_FRAME_MAX_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES,
    worth_store_physical_format::MAX_CHECKPOINT_BINDING_RECORD_BYTES,
};

#[test]
fn one_head_recovery_scope_rejects_before_manifest_wal_or_root_effect() {
    std::thread::Builder::new()
        .name("release-capacity-low-recovery".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let limit = numeric_one_head_closure();
            let (world, proof, publication_record) =
                worth_store_testkit_recovery_runtime::production_entry::release_reopen::published_world::create(false, None, std::num::NonZeroU64::new(limit));
            let serving = world.serving();
            let policy = serving.residency_observation().admitted_policy();
            assert_eq!(policy.operation_bytes(), 32 << 20);
            assert_eq!(
                policy.scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
                limit,
            );
            let before = snapshot_family(world.root());
            let counters = serving.media_counters();
            let selected = serving
                .certification_selected_latest_blob_publication()
                .unwrap()
                .expect("lawfully published source remains selected");
            let result = serving
                .blobs()
                .unwrap()
                .reclaim(release_request(proof, world.placement()))
                .and_then(|handle| handle.wait());
            match result {
                Err(worth_store::physical_runtime::BlobReclaimFailure::ReleaseCertificateBacking(
                    worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
                )) => {
                    assert_eq!(admitted, limit);
                    assert!(required > admitted);
                }
                Err(other) => panic!("pre-effect release capacity denial: {other:?}"),
                Ok(_) => panic!("release unexpectedly completed at numeric-only closure"),
            }
            assert_eq!(snapshot_family(world.root()), before);
            let after = serving.media_counters();
            assert_eq!(after.append_attempts(), counters.append_attempts());
            assert_eq!(after.replacements(), counters.replacements());
            assert_eq!(after.file_creates(), counters.file_creates());
            assert_eq!(
                serving
                    .certification_selected_latest_blob_publication()
                    .unwrap()
                    .unwrap()
                    .record(),
                selected.record(),
            );
            assert_eq!(selected.record(), publication_record);
            world.close();
        })
        .expect("release capacity worker")
        .join()
        .expect("release capacity worker did not panic");
}

#[test]
fn sufficient_recovery_scope_drops_then_checkpoints_and_reopens_with_recovery_seal() {
    std::thread::Builder::new()
        .name("release-capacity-sufficient-recovery".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let (world, proof, record) =
                worth_store_testkit_recovery_runtime::production_entry::release_reopen::published_world::create(false, None, std::num::NonZeroU64::new(32 << 20));
            assert_eq!(
                world
                    .serving()
                    .residency_observation()
                    .admitted_policy()
                    .scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
                32 << 20,
            );
            let receipt = world
                .serving()
                .blobs()
                .unwrap()
                .reclaim(release_request(proof, world.placement()))
                .expect("sufficient Recovery scope admits release")
                .wait()
                .expect("sufficient Recovery scope completes release");
            assert_eq!(receipt.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
            assert_eq!(receipt.remaining_payload_records(), 0);
            assert!(receipt.dropped_records().contains(&record));
            worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run_world((world, receipt, record), worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::None);
        })
        .expect("release capacity worker")
        .join()
        .expect("release capacity worker did not panic");
}
#[cfg(all())]
#[path = "checkpoint_funding/mod.rs"]
pub(crate) mod checkpoint_funding;
#[cfg(all())]
#[path = "checkpoint_overlap/mod.rs"]
pub(crate) mod checkpoint_overlap;
#[cfg(all())]
#[path = "checkpoint_read_funding/mod.rs"]
pub(crate) mod checkpoint_read_funding;
#[cfg(all())]
#[path = "live_admission/mod.rs"]
pub(crate) mod live_admission;
#[cfg(all())]
#[path = "recovery_pool_handoff/mod.rs"]
pub(crate) mod recovery_pool_handoff;
#[cfg(all())]
#[path = "source_read_funding/mod.rs"]
pub(crate) mod source_read_funding;
