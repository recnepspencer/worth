//! An artifact longer than its own ceiling is damage whatever recovery has
//! left of the budget it is read under: a selector past its fixed length, a
//! manifest past its page with fewer observation bytes left than it holds,
//! a routing block past its page with exactly one page of manifest bytes
//! left, a head block past its page. The checkpoint stream has no ceiling of
//! its own, so the bytes recovery may observe are what a longer one passes.

pub(crate) use worth_store_recovery_runtime::PhysicalRecoveryLimitDimension;
pub(crate) use worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::PendingWalWorld;
use worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::{Tail, Workload};
pub(crate) use {
    crate::production_entry::published_above_checkpoint,
    crate::production_entry::published_above_checkpoint::damaged_history::recover_with_longer,
    crate::production_entry::published_above_checkpoint::damaged_history::recover_with_oversized,
    crate::production_entry::published_above_checkpoint::damaged_history::root_manifest,
    crate::production_entry::published_above_checkpoint::damaged_history::routing_blocks,
    crate::production_entry::published_above_checkpoint::damaged_history::PAST_ONE_PAGE, std::fs,
    std::path::Path, std::path::PathBuf, worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history,
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world,
};

/// Blocked without effect and without naming a limit; `false` where the
/// recovery never read the artifact and recovered.
pub(crate) fn blocks_as_damage(outcome: PhysicalRecoveryOutcome, artifact: &Path) -> bool {
    match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => {
            drop(handoff);
            false
        }
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => {
            assert_eq!(blocked.recovery_effects(), 0);
            let evidence = blocked.evidence();
            assert!(
                blocked.cause().limit().is_none(),
                "{artifact:?} oversized is damage: {:?} {:?}",
                blocked.cause(),
                evidence.planning_denial,
            );
            true
        }
        outcome => panic!("{artifact:?} oversized: {outcome:?}"),
    }
}

pub(crate) fn idle_world(
) -> worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::PendingWalWorld {
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle)
}

/// Every file under `directory` whose name ends in `.manifest`.
pub(crate) fn manifests(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            manifests(&path, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "manifest")
        {
            found.push(path);
        }
    }
}

/// The observation bytes of these recoveries, and how much longer each
/// manifest is made: one byte more than recovery may observe in all.
pub(crate) const OBSERVATION_BYTES: u64 = 4 << 20;

pub(crate) fn observing(root: &Path) -> PhysicalRecoveryOpenRequest {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_narrowing(root, |declared| {
        declared.observation_bytes = OBSERVATION_BYTES
    })
}

pub(crate) fn under_manifest_bytes(
    root: &Path,
    manifest_bytes: u64,
) -> PhysicalRecoveryOpenRequest {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_narrowing(root, |declared| {
        declared.manifest_bytes = manifest_bytes
    })
}

/// The manifest-byte limit this block names, if it names that: what the
/// read reached, and what recovery admitted.
pub(crate) fn manifest_byte_limit(outcome: &PhysicalRecoveryOutcome) -> Option<(u64, u64)> {
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("too few manifest bytes must block: {outcome:?}")
    };
    blocked
        .cause()
        .limit()
        .filter(|limit| {
            limit.dimension()
                == worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ManifestBytes
        })
        .map(|limit| (limit.observed(), limit.admitted()))
}

/// The head blocks the world holds, in name order.
pub(crate) fn head_blocks(root: &Path) -> Vec<PathBuf> {
    let mut all = Vec::new();
    manifests(&root.join("families/records"), &mut all);
    all.retain(|path| {
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("release-head-"))
    });
    all.sort();
    all
}

/// A head block is one page of its format. One byte past that page with
/// every byte admitted, and longer than every observation byte recovery
/// admits, it is damage and names no limit. Each block is damaged in the
/// world as the kill left it; one this recovery never reads recovers.
pub(crate) fn assert_an_oversized_head_block_is_damage(
    killed: impl Fn() -> PendingWalWorld,
    stage: &str,
) {
    let count = head_blocks(killed().root()).len();
    assert!(count > 0, "{stage}: the world holds a head block");
    let mut read = 0;
    for index in 0..count {
        let world = killed();
        let blocks = head_blocks(world.root());
        assert_eq!(blocks.len(), count, "{stage}: the kill is deterministic");
        let block = &blocks[index];
        if !blocks_as_damage(recover_with_oversized(world.root(), block), block) {
            continue;
        }
        let past_every_byte = recover_with_longer(block, OBSERVATION_BYTES as usize + 1, || {
            observing(world.root())
        });
        assert!(
            blocks_as_damage(past_every_byte, block),
            "{stage}: {block:?} is read under the same observation bytes",
        );
        read += 1;
        // The block is back as the kill left it, and the same limits recover.
        let outcome =
            worth_store_recovery_runtime::WorthStoreRecovery::recover(observing(world.root()));
        assert!(
            matches!(
                outcome,
                worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(_)
            ),
            "{stage}: the undamaged store recovers: {outcome:?}",
        );
    }
    assert!(
        read > 0,
        "{stage}: recovery reads none of {count} head blocks"
    );
}
