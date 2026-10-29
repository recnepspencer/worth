//! A completed-history seal must freshly inspect both selected head and
//! ordinary-ingest control media before it can become Serving.

pub(crate) use {std::path::Path, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_physical_format::RecordArtifactFile, worth_store_recovery_runtime::WorthStoreRecovery, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, std::fs, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, crate::production_entry::historical_release_only::selected_generation};
pub(crate) use {std::path::PathBuf};
pub(crate) use {worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody};
pub(crate) use {worth_store_physical_format::DurablePhysicalRootManifest};

pub(crate) fn selected_head_block(root: &Path) -> PathBuf {
    let roots = root.join("families/records/roots");
    let bytes = std::fs::read(
        roots.join(
            worth_store_physical_format::RecordArtifactFile::RootManifest {
                generation: selected_generation(root),
            }
            .file_name(),
        ),
    )
    .unwrap();
    let (manifest, _) = worth_store_physical_format::DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    let head = manifest
        .release_custody_head_root()
        .expect("actual selected completed head tree");
    roots.join(
        worth_store_physical_format::RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: head.generation(),
            block: head.block(),
        }
        .file_name(),
    )
}

pub(crate) fn fresh_completed_seal(root: &Path) -> RecoveredPhysicalCheckpointCustody {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("unchanged completed history must freshly recover: {outcome:?}")
    };
    assert_eq!(handoff.core().recovery_effect_count(), 0);
    handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("completed history must freshly produce a Store seal")
}

pub(crate) fn deny_changed_media_then_retry(root: &Path, path: &Path, byte: usize, subject: &str) {
    let original = std::fs::read(path).unwrap();
    assert!(
        byte < original.len(),
        "{subject} byte must belong to actual media"
    );
    let seal = fresh_completed_seal(root);
    let allocations = seal.certification_residency_allocations();
    let recovery =
        worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
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
    std::fs::write(path, &changed).unwrap();
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_expect_mismatch(root, seal);
    assert_eq!(
        std::fs::read(path).unwrap(),
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

    std::fs::write(path, &original).unwrap();
    let seal = fresh_completed_seal(root);
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal).close();
}