//! Ordering the retirements above the checkpoint reads the root manifest and
//! routing blocks of every generation among them. One that outgrew its page
//! is damaged media. Recovery must say so and must not name a limit: an
//! operator would raise a limit on a corrupt store.
//!
//! A limit comes from a budget the caller set. No artifact's own ceiling is
//! one, and neither is a length read off media.

pub(crate) use {std::path::Path, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, crate::production_entry::published_above_checkpoint};
pub(crate) use {};
pub(crate) use {std::fs};
pub(crate) use {std::path::PathBuf};
pub(crate) use {worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryOpenRequest, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryPageAdmissionDenial, worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial, worth_store_recovery_runtime::WorthStoreRecovery};



/// More than one page: no root manifest or routing block is this large.
pub(crate) const PAST_ONE_PAGE: usize = (1 << 16) + 1;

pub(crate) fn root_manifest(roots: &Path, generation: u64) -> PathBuf {
    roots.join(worth_store_physical_format::RecordArtifactFile::RootManifest { generation }.file_name())
}

/// The routing blocks written with the root of `generation`.
pub(crate) fn routing_blocks(roots: &Path, generation: u64) -> Vec<PathBuf> {
    (1..=generation)
        .map(|block| {
            roots.join(worth_store_physical_format::RecordArtifactFile::RootRoutingBlock { generation, block }.file_name())
        })
        .filter(|path| path.exists())
        .collect()
}

/// Recovery over the store while `artifact` is `PAST_ONE_PAGE` bytes longer.
pub(crate) fn recover_with_oversized(root: &Path, artifact: &Path) -> PhysicalRecoveryOutcome {
    recover_with_longer(artifact, PAST_ONE_PAGE, || {
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_manifest_entries(root, 4096)
    })
}

/// What `request` recovers while `artifact` is `extra` zero bytes longer.
pub(crate) fn recover_with_longer(
    artifact: &Path,
    extra: usize,
    request: impl FnOnce() -> PhysicalRecoveryOpenRequest,
) -> PhysicalRecoveryOutcome {
    let original = std::fs::read(artifact).unwrap();
    let mut longer = original.clone();
    longer.resize(original.len() + extra, 0);
    std::fs::write(artifact, &longer).unwrap();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(request());
    std::fs::write(artifact, &original).unwrap();
    outcome
}