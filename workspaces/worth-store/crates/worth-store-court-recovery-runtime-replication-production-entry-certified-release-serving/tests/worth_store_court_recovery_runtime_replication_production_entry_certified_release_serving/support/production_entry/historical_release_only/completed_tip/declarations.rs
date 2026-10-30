//! Two completed releases rejoin without an ordinary tail or new checkpoint,
//! then retain exact per-object progress across checkpoint and fresh recovery.

pub(crate) use {
    crate::production_entry::historical_release_only::selected_generation, std::fs,
    std::path::Path, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};
pub(crate) use {
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::ReleaseCustodyHeadEntryV1,
};
pub(crate) use {
    worth_store_physical_integrity::walk_release_custody_head,
    worth_store_physical_integrity::ReleaseCustodyHeadWalkLimitsV1,
};

pub(crate) fn checkpoint_completed_history(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xc9; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("completed-history Serving checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "completed-history checkpoint must complete: {outcome:?}"
    );
}

pub(crate) fn selected_root_heads(root: &Path) -> Vec<ReleaseCustodyHeadEntryV1> {
    let records = root.join("families/records/roots");
    let bytes = std::fs::read(
        records.join(
            worth_store_physical_format::RecordArtifactFile::RootManifest {
                generation: selected_generation(root),
            }
            .file_name(),
        ),
    )
    .unwrap();
    let (manifest, format) =
        worth_store_physical_format::DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    let limits = worth_store_physical_integrity::ReleaseCustodyHeadWalkLimitsV1::new(
        32,
        2,
        32 * u64::from(format.page_size().bytes()),
        1 << 20,
        16,
    )
    .unwrap();
    let mut entries = Vec::new();
    let walk = walk_release_custody_head(
        &manifest,
        format,
        limits,
        |reference, remaining| {
            let bytes = std::fs::read(
                records.join(
                    worth_store_physical_format::RecordArtifactFile::ReleaseCustodyHeadBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
                    .file_name(),
                ),
            )?;
            assert!(bytes.len() as u64 <= remaining);
            Ok::<_, std::io::Error>(bytes)
        },
        |entry| {
            entries.push(entry);
            Ok::<_, ()>(())
        },
    )
    .expect("independent selected-media head walk");
    assert_eq!(walk.entry_count(), 2);
    entries
}
