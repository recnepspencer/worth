//! An artifact longer than its own ceiling is damage whatever recovery has
//! left of the budget it is read under: a selector past its fixed length, a
//! manifest past its page with fewer observation bytes left than it holds,
//! a routing block past its page with exactly one page of manifest bytes
//! left, a head block past its page. The checkpoint stream has no ceiling of
//! its own, so the bytes recovery may observe are what a longer one passes.

use super::*;
use pending_wal_world::PendingWalWorld;
use worth_store_recovery_runtime::PhysicalRecoveryLimitDimension;

/// Blocked without effect and without naming a limit; `false` where the
/// recovery never read the artifact and recovered.
fn blocks_as_damage(outcome: PhysicalRecoveryOutcome, artifact: &Path) -> bool {
    match outcome {
        PhysicalRecoveryOutcome::Recovered(handoff) => {
            drop(handoff);
            false
        }
        PhysicalRecoveryOutcome::Blocked(blocked) => {
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

fn idle_world() -> pending_wal_world::PendingWalWorld {
    pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle)
}

/// A selector is a fixed 107 bytes. That is its own length, not a limit: a
/// longer one is rejected as any damaged selector is.
#[test]
fn a_selector_longer_than_a_selector_is_damage_not_a_limit() {
    // Nothing selects a root without the current selector.
    let world = idle_world();
    let current = world.root().join("families/records/root-current.selector");
    let outcome = recover_with_oversized(world.root(), &current);
    assert!(
        blocks_as_damage(outcome, &current),
        "the current one is read"
    );
    let serving = super::super::serve(&world, "after the damaged attempt");
    world.assert_objects_read_back(&serving);
    serving.close();

    // The current selector still selects where the previous one is rejected.
    // That recovery publishes, so the selector is left as recovery leaves it.
    let world = idle_world();
    let previous = world.root().join("families/records/root-previous.selector");
    let mut longer = fs::read(&previous).unwrap();
    longer.resize(longer.len() + PAST_ONE_PAGE, 0);
    fs::write(&previous, &longer).unwrap();
    let outcome = WorthStoreRecovery::recover(
        certified_release_serving::request_with_manifest_entries(world.root(), 4096),
    );
    assert!(
        matches!(outcome, PhysicalRecoveryOutcome::Recovered(_)),
        "a rejected previous selector stops nothing: {outcome:?}",
    );
    drop(outcome);
    let serving = super::super::serve(&world, "after the rejected previous selector");
    world.assert_objects_read_back(&serving);
    serving.close();
}

/// Every file under `directory` whose name ends in `.manifest`.
fn manifests(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
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
const OBSERVATION_BYTES: u64 = 4 << 20;

fn observing(root: &Path) -> PhysicalRecoveryOpenRequest {
    certified_release_serving::request_narrowing(root, |declared| {
        declared.observation_bytes = OBSERVATION_BYTES
    })
}

/// No manifest artifact is longer than one page, so one longer than every
/// observation byte recovery admits did not run recovery out of them. Root
/// manifests, routing blocks, free-space manifests and blocks, and segment
/// blocks are each read under their own page.
#[test]
fn a_manifest_longer_than_the_observation_bytes_is_damage_not_a_limit() {
    let world = idle_world();
    let records = world.root().join("families/records");
    let mut all = Vec::new();
    manifests(&records, &mut all);
    all.sort();
    let mut blocked = std::collections::BTreeMap::<String, usize>::new();
    for manifest in &all {
        let outcome = recover_with_longer(manifest, OBSERVATION_BYTES as usize + 1, || {
            observing(world.root())
        });
        if blocks_as_damage(outcome, manifest) {
            let family = manifest.parent().unwrap().file_name().unwrap();
            *blocked
                .entry(family.to_string_lossy().into_owned())
                .or_default() += 1;
        }
    }
    for family in ["roots", "free-space", "segment-manifests"] {
        assert!(
            blocked.get(family).is_some_and(|count| *count > 0),
            "recovery reads a manifest of {family}: {blocked:?} of {}",
            all.len(),
        );
    }
    // The same limits recover the store as the kill left it.
    let PhysicalRecoveryOutcome::Recovered(handoff) =
        WorthStoreRecovery::recover(observing(world.root()))
    else {
        panic!("the undamaged store recovers under the same observation bytes")
    };
    drop(handoff);
    let serving = super::super::serve(&world, "after the damaged attempts");
    world.assert_objects_read_back(&serving);
    serving.close();
}

fn under_manifest_bytes(root: &Path, manifest_bytes: u64) -> PhysicalRecoveryOpenRequest {
    certified_release_serving::request_narrowing(root, |declared| {
        declared.manifest_bytes = manifest_bytes
    })
}

/// The manifest-byte limit this block names, if it names that: what the
/// read reached, and what recovery admitted.
fn manifest_byte_limit(outcome: &PhysicalRecoveryOutcome) -> Option<(u64, u64)> {
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("too few manifest bytes must block: {outcome:?}")
    };
    blocked
        .cause()
        .limit()
        .filter(|limit| limit.dimension() == PhysicalRecoveryLimitDimension::ManifestBytes)
        .map(|limit| (limit.observed(), limit.admitted()))
}

/// A routing block is read under its page or the manifest bytes recovery has
/// left, whichever is less. One byte past its page it is damage at every
/// count left, exactly one page among them. Exactly its page long it is
/// within its ceiling, so too few bytes for it are still the limit.
#[test]
fn an_oversized_routing_block_is_damage_whatever_manifest_bytes_are_left() {
    let world = idle_world();
    let roots = world.root().join("families/records/roots");
    let selected = (1..=4096)
        .rev()
        .find(|generation| root_manifest(&roots, *generation).exists())
        .expect("a published root");
    let block = routing_blocks(&roots, selected).remove(0);
    let length = fs::metadata(&block).unwrap().len();
    // The page is the one the selected root declares.
    let (_, format) = worth_store_physical_format::DurablePhysicalRootManifest::decode(
        &fs::read(root_manifest(&roots, selected)).unwrap(),
        u16::MAX,
    )
    .unwrap();
    let page = u64::from(format.page_size().bytes());
    assert!(length < page, "the block leaves room to pad");
    let to_page = (page - length) as usize;
    let limit = |observed, admitted| Some((observed, admitted));
    let as_written = |manifest_bytes| {
        WorthStoreRecovery::recover(under_manifest_bytes(world.root(), manifest_bytes))
    };
    let longer_by = |extra, manifest_bytes| {
        recover_with_longer(&block, extra, || {
            under_manifest_bytes(world.root(), manifest_bytes)
        })
    };
    // Each limit names the count its read reached, so the limits of the
    // store as written lead from read to read. The block is read at the
    // first count where one byte past its page changes the answer.
    let mut before = 1;
    let mut reads = 0;
    let next = loop {
        let next = manifest_byte_limit(&as_written(before)).expect("short of every manifest");
        assert_eq!(next.1, before);
        if manifest_byte_limit(&longer_by(to_page + 1, before)) != Some(next) {
            break next;
        }
        before = next.0;
        reads += 1;
        assert!(reads < 64, "the block is never read");
    };
    assert!(reads > 0, "a root is read before its block");
    assert_eq!(
        Some(next),
        limit(before + length, before),
        "the block as written"
    );

    // One byte past its page the block is damage. Recovery goes on without
    // that root: no limit it then names is this block's length, and with a
    // page or so left it names none.
    for left in [0, 1, page - 1, page, page + 1] {
        let outcome = longer_by(to_page + 1, before + left);
        let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
            panic!("{left} left: {outcome:?}")
        };
        assert_eq!(blocked.recovery_effects(), 0);
        let named = blocked.cause().limit();
        assert_ne!(
            named.map(|limit| limit.observed()),
            Some(before + page + 1),
            "{left} manifest bytes left: {named:?}",
        );
        if left + 1 >= page {
            assert_eq!(
                (blocked.cause().damage(), named),
                (Some(PhysicalRecoveryBlockKind::SourceSelection), None),
                "{left} manifest bytes left",
            );
        }
    }
    // Exactly its page long the block is within its own ceiling, so one
    // manifest byte short of that page is the limit, with the block's length.
    assert_eq!(
        manifest_byte_limit(&longer_by(to_page, before + page - 1)),
        limit(before + page, before + page - 1)
    );
    let serving = super::super::serve(&world, "after the damaged attempts");
    world.assert_objects_read_back(&serving);
    serving.close();
}

/// Nothing declares the checkpoint stream's length, so one longer than every
/// observation byte recovery admits ran recovery out of them. The limit
/// counts from the first byte discovery read: the selectors, roots and
/// routing blocks before the stream are in it.
#[test]
fn a_checkpoint_stream_longer_than_the_observation_bytes_is_that_limit_from_the_first_byte() {
    let world = idle_world();
    let stream = world.root().join("families/checkpoint.current");
    let length = fs::metadata(&stream).unwrap().len() + OBSERVATION_BYTES + 1;
    let outcome = recover_with_longer(&stream, OBSERVATION_BYTES as usize + 1, || {
        observing(world.root())
    });
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("a stream past every observation byte must block: {outcome:?}")
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let evidence = blocked.evidence();
    let before = evidence.counters.bytes_observed;
    assert!(before > 0, "discovery reads roots before the stream");
    assert_eq!(
        blocked.cause().limit().map(|limit| (
            limit.dimension(),
            limit.observed(),
            limit.admitted()
        )),
        Some((
            PhysicalRecoveryLimitDimension::ObservationBytes,
            before + length,
            OBSERVATION_BYTES,
        )),
    );
    let serving = super::super::serve(&world, "after the oversized stream");
    world.assert_objects_read_back(&serving);
    serving.close();
}

/// The head blocks the world holds, in name order.
fn head_blocks(root: &Path) -> Vec<PathBuf> {
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
fn assert_an_oversized_head_block_is_damage(killed: impl Fn() -> PendingWalWorld, stage: &str) {
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
        let outcome = WorthStoreRecovery::recover(observing(world.root()));
        assert!(
            matches!(outcome, PhysicalRecoveryOutcome::Recovered(_)),
            "{stage}: the undamaged store recovers: {outcome:?}",
        );
    }
    assert!(
        read > 0,
        "{stage}: recovery reads none of {count} head blocks"
    );
}

#[test]
fn a_head_block_longer_than_its_page_is_damage_not_a_limit_where_a_release_completed() {
    assert_an_oversized_head_block_is_damage(
        super::super::manifest_entry_limit::released_world,
        "released above the checkpoint",
    );
}

#[test]
fn a_head_block_longer_than_its_page_is_damage_not_a_limit_above_a_checkpoint_head() {
    assert_an_oversized_head_block_is_damage(
        pending_successor_above_history::terminal_successor_of_a_checkpoint_head,
        "released above a checkpoint head",
    );
}

/// A release that completed above the checkpoint's heads is replayed from
/// the history walk, which reads the head block before the gate does.
#[test]
fn a_head_block_longer_than_its_page_is_damage_not_a_limit_under_a_completed_history() {
    assert_an_oversized_head_block_is_damage(
        pending_successor_above_history::successor_of_a_checkpoint_head_above_foreign_history,
        "completed above the checkpoint heads",
    );
    assert_an_oversized_head_block_is_damage(
        pending_successor_above_history::ordered_release_above_a_head_checkpoint,
        "ordered above a head checkpoint",
    );
}
