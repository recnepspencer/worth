//! What the generation-zero basis must never admit: a store whose media
//! shows something only a checkpoint could have made, or whose checkpoint is
//! present but unreadable.

pub(crate) use worth_store::physical_runtime::RecordBootstrapDenial;
pub(crate) use {
    worth_store_physical_format::durable_artifact_checksum,
    worth_store_physical_format::DurableFreeSpaceManifestHeader,
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::DurableRootSelector,
    worth_store_physical_format::RecordArtifactFile,
};
pub(crate) use {
    worth_store_recovery_runtime::HistoricalDropAdmissionStage,
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryPageAdmissionDenial,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
};

pub(crate) use {
    crate::production_entry::before_first_checkpoint::FirstWrite,
    crate::production_entry::before_first_checkpoint::KilledWorld, std::fs, std::path::Path,
    worth_proof::TransitionOutcome, worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};

/// Recovery must block in `phase`, with the planning denial `denial` admits,
/// never open the generation-zero basis.
pub(crate) fn assert_blocked(
    root: &Path,
    stage: &str,
    phase: PhysicalRecoveryBlockKind,
    denial: fn(Option<&PhysicalRecoveryPlanningDenial>) -> bool,
) {
    match worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root)) {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => assert!(
            block.cause().damage() == Some(phase)
                && denial(block.evidence().planning_denial.as_ref()),
            "{stage} blocked elsewhere: kind={:?}; cause={:?}",
            block.cause(),
            block.evidence().planning_denial
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_) => panic!("{stage} opened without a checkpoint"),
        _ => panic!("{stage} neither blocked nor recovered"),
    }
}

/// Rewrites the selected root's free-space header to start a tier epoch, and
/// rebinds the root to the rewritten header so only the epoch is wrong.
pub(crate) fn start_a_tier_epoch_in_the_free_space_header(root: &Path) {
    let records = root.join("families/records");
    let selector = std::fs::read(
        records
            .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name()),
    )
    .expect("current root selector");
    let generation = worth_store_physical_format::DurableRootSelector::decode(&selector)
        .unwrap()
        .root_generation();
    let root_path = records.join("roots").join(
        worth_store_physical_format::RecordArtifactFile::RootManifest { generation }.file_name(),
    );
    let (manifest, format) = worth_store_physical_format::DurablePhysicalRootManifest::decode(
        &std::fs::read(&root_path).unwrap(),
        u16::MAX,
    )
    .unwrap();
    assert!(manifest.tier_epoch_anchor().is_none());
    let header_path = records.join("free-space").join(
        worth_store_physical_format::RecordArtifactFile::FreeSpaceManifest { generation }
            .file_name(),
    );
    let (header, _) = worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(
        &std::fs::read(&header_path).unwrap(),
        u16::MAX,
    )
    .unwrap();
    assert!(header.tier_epoch_start().is_none());
    let started = worth_store_physical_format::DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        header.generation(),
        header.tree_identity(),
        header.node_capacity(),
        header.segment_page_capacity(),
        header.entry_count(),
        header.next_segment(),
        header.next_page(),
        header.next_extent(),
        header.next_arena(),
        Some(header.next_arena()),
        header.arena_capacity(),
        header.arena_alignment(),
        header.next_block(),
        header.root(),
    )
    .expect("a structurally valid tier epoch start")
    .encode(format);
    std::fs::write(&header_path, &started).unwrap();
    let rebound = worth_store_physical_format::DurablePhysicalRootManifest::builder(
        manifest.generation(),
        manifest.tree_identity(),
        manifest.node_capacity(),
        durable_artifact_checksum(&started),
    )
    .record_count(manifest.record_count())
    .next_block(manifest.next_block())
    .next_segment_block(manifest.next_segment_block())
    .routing_root(manifest.routing_root())
    .segment_root(manifest.segment_root())
    .free_space_root(manifest.free_space_root())
    .last_inline_record(manifest.last_inline_record())
    .last_inline_segment(manifest.last_inline_segment())
    .admit()
    .unwrap();
    std::fs::write(&root_path, rebound.encode(format)).unwrap();
}
