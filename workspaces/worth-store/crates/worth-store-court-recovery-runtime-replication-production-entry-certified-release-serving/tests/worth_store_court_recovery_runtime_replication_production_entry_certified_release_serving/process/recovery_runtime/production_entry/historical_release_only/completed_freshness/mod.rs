//! A completed-history seal must freshly inspect both selected head and
//! ordinary-ingest control media before it can become Serving.

#[allow(unused_imports)]
use crate::production_entry::historical_release_only::completed_freshness::*;
use std::path::PathBuf;
use worth_store_physical_format::DurablePhysicalRootManifest;
use {
    crate::production_entry::historical_release_only::selected_generation, std::fs,
    std::path::Path, worth_store_physical_format::RecordArtifactFile,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world,
};
use {
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalResidencyDimension,
    worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
};

#[test]
fn completed_history_seal_denies_changed_head_and_ordinary_control_then_freshly_retries() {
    std::thread::Builder::new()
        .name("completed-history-media-freshness".into())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
            world.kill_distinct_release_before_checkpoint();
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
                worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()))
            else {
                panic!("real pending second release must recover")
            };
            assert!(handoff.core().recovery_effect_count() > 0);
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .unwrap();
            worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal).close();
            let selected = selected_generation(world.root());

            let head_path = selected_head_block(world.root());
            let head_bytes = std::fs::read(&head_path).unwrap();
            assert!(!head_bytes.is_empty());
            deny_changed_media_then_retry(
                world.root(),
                &head_path,
                head_bytes.len() / 2,
                "selected completed release head",
            );

            let range = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::selected_session_declared_extent(world.root());
            let arena_path = world.root().join("families/records/arenas").join(
                worth_store_physical_format::RecordArtifactFile::ExtentArena {
                    arena: range.arena().get(),
                }
                .file_name(),
            );
            assert!(range.length() > 16);
            let manifest_byte = usize::try_from(range.offset() + 16).unwrap();
            deny_changed_media_then_retry(
                world.root(),
                &arena_path,
                manifest_byte,
                "selected ordinary SessionDeclared extent",
            );
            assert_eq!(selected_generation(world.root()), selected);
        })
        .unwrap()
        .join()
        .expect("completed-history media freshness worker");
}
