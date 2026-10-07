//! A completed-history seal must freshly inspect both selected head and
//! ordinary-ingest control media before it can become Serving.

use super::*;
use std::path::PathBuf;
use worth_store::physical_runtime::{
    PhysicalOperationAllocationScope, PhysicalResidencyDimension,
    RecoveredPhysicalCheckpointCustody,
};
use worth_store_physical_format::DurablePhysicalRootManifest;

#[test]
fn completed_history_seal_denies_changed_head_and_ordinary_control_then_freshly_retries() {
    std::thread::Builder::new()
        .name("completed-history-media-freshness".into())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = pending_wal_world::first();
            world.kill_distinct_release_before_checkpoint();
            let PhysicalRecoveryOutcome::Recovered(handoff) =
                WorthStoreRecovery::recover(certified_release_serving::request(world.root()))
            else {
                panic!("real pending second release must recover")
            };
            assert!(handoff.core().recovery_effect_count() > 0);
            let seal = handoff.into_core().into_checkpoint_custody().unwrap();
            certified_release_serving::admit_serving_with_seal(world.root(), seal).close();
            let selected = selected_generation(world.root());

            let head_path = selected_head_block(world.root());
            let head_bytes = fs::read(&head_path).unwrap();
            assert!(!head_bytes.is_empty());
            deny_changed_media_then_retry(
                world.root(),
                &head_path,
                head_bytes.len() / 2,
                "selected completed release head",
            );

            let range = pending_wal_world::selected_session_declared_extent(world.root());
            let arena_path = world.root().join("families/records/arenas").join(
                RecordArtifactFile::ExtentArena {
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

fn selected_head_block(root: &Path) -> PathBuf {
    let roots = root.join("families/records/roots");
    let bytes = fs::read(
        roots.join(
            RecordArtifactFile::RootManifest {
                generation: selected_generation(root),
            }
            .file_name(),
        ),
    )
    .unwrap();
    let (manifest, _) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    let head = manifest
        .release_custody_head_root()
        .expect("actual selected completed head tree");
    roots.join(
        RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: head.generation(),
            block: head.block(),
        }
        .file_name(),
    )
}

fn fresh_completed_seal(root: &Path) -> RecoveredPhysicalCheckpointCustody {
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("unchanged completed history must freshly recover: {outcome:?}")
    };
    assert_eq!(handoff.core().recovery_effect_count(), 0);
    handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("completed history must freshly produce a Store seal")
}

fn deny_changed_media_then_retry(root: &Path, path: &Path, byte: usize, subject: &str) {
    let original = fs::read(path).unwrap();
    assert!(
        byte < original.len(),
        "{subject} byte must belong to actual media"
    );
    let seal = fresh_completed_seal(root);
    let allocations = seal.certification_residency_allocations();
    let recovery =
        PhysicalResidencyDimension::OperationScope(PhysicalOperationAllocationScope::Recovery);
    assert!(
        allocations
            .snapshot()
            .for_dimension(recovery)
            .active_units()
            > 0,
        "completed history must retain native backing in its seal",
    );
    let mut changed = original.clone();
    changed[byte] ^= 0x40;
    fs::write(path, &changed).unwrap();
    certified_release_serving::open_serving_with_seal_expect_mismatch(root, seal);
    assert_eq!(
        fs::read(path).unwrap(),
        changed,
        "denial must not rewrite {subject}"
    );
    let disposed = allocations.snapshot().for_dimension(recovery);
    assert_eq!(
        disposed.active_units(),
        0,
        "{subject} denial must dispose backing"
    );
    assert_eq!(disposed.admitted_units(), disposed.released_units());

    fs::write(path, &original).unwrap();
    let seal = fresh_completed_seal(root);
    certified_release_serving::admit_serving_with_seal(root, seal).close();
}
