//! Layer-3 symbol-dictionary complexity court (RULES §4, public #446). A
//! single-record commit reserves its client key's symbol in the candidate
//! and publishes it to the owner dictionary: `O(|touched|)` dictionary
//! writes, each an `O(log N)` persistent path copy that shares every
//! existing client key's text. The binary's global allocator attributes
//! every allocation of the commit by stack (`allocation_attribution`).
//!
//! At 1x and 10x existing client keys, within one persistent-map node
//! (6 -> 60 keys, where a deep-copying text carrier pays one copy per sibling
//! key) and across tree levels (64 -> 640), in a plain world and in one with
//! persistent names, Anchor groups and a forked branch composed:
//! - no symbol text is copied and no whole dictionary is materialized;
//! - the whole commit outside the storage tree stays within 1.1x;
//! - across tree levels the dictionary's share stays within 1.1x. Within a
//!   node it is reported: the randomly seeded hash map may add one path level
//!   by 60 keys, an `O(log N)` step the bound does not describe.
//! The storage tree's record-column path copies deepen with the record
//! count; they are reported. No production counter is involved.
use super::allocation_attribution::{measure, ratio, Allocations, Share};
use super::composed_world::{self, World};
use worth_relational::facade::inspection::observe_symbol_snapshot_materialization;
use worth_relational::facade::lineage::CommitNamingPlan;

/// Existing client keys at 1x within one node and across tree levels; 10x
/// is ten times as many.
const WITHIN_NODE: usize = 6;
const ACROSS_LEVELS: usize = 64;
const BATCH: usize = 64;

/// `count` client keys, in commits of at most 64.
fn batches(count: usize) -> Vec<Vec<String>> {
    let keys: Vec<_> = (0..count)
        .map(|index| format!("existing.{index}"))
        .collect();
    keys.chunks(BATCH).map(<[_]>::to_vec).collect()
}

/// A world whose dictionary holds `existing` client keys of plain records.
fn plain(existing: usize) -> World {
    let world = World::unnamed();
    for batch in batches(existing) {
        let creates = batch
            .iter()
            .map(|key| composed_world::create(key))
            .collect();
        world.commit(&world.main(), creates, CommitNamingPlan::default());
    }
    world
}

/// A world holding `existing` client keys born with persistent names,
/// grouped in eights, a sixteenth of them retired, with a forked branch.
fn composed(existing: usize) -> World {
    let world = World::new();
    let mut names = Vec::new();
    for batch in batches(existing) {
        world.authored(&world.main(), &batch);
        names.extend(batch.iter().map(|key| World::authored_name(key)));
    }
    world.group(&world.main(), &names, 8);
    for name in names.iter().step_by(16) {
        world.retire(&world.main(), *name);
    }
    world.fork("side");
    world
}

/// One commit creating one record under a new client key: unnamed in the
/// plain world, born with a persistent name in the composed one.
fn one_record_commit(world: &World, named: bool) -> Allocations {
    let basis = world.main();
    let (measured, materialized) = observe_symbol_snapshot_materialization(|| {
        if named {
            measure(|| world.authored(&basis, &["measured".to_owned()]))
        } else {
            let creates = vec![composed_world::create("measured")];
            measure(|| world.commit(&basis, creates, CommitNamingPlan::default()))
        }
    });
    assert_eq!(
        materialized, 0,
        "a commit never materializes the dictionary"
    );
    assert_eq!(
        measured.text_copies.count, 0,
        "a commit never copies a symbol text: {measured:?}"
    );
    assert!(
        measured.symbols.count > 0,
        "the commit's dictionary work is attributed: stacks carry symbols"
    );
    measured
}

fn assert_share_flat(large: Share, small: Share, axis: &str) {
    let calls = ratio(large.count, small.count);
    let bytes = ratio(large.bytes, small.bytes);
    println!("{axis}: calls x{calls:.3}, bytes x{bytes:.3}");
    assert!(
        calls <= 1.1 && bytes <= 1.1,
        "{axis}: {large:?} vs {small:?} (calls x{calls:.3}, bytes x{bytes:.3})"
    );
}

fn assert_flat_in_existing_client_keys(build: fn(usize) -> World, named: bool, label: &str) {
    for one in [WITHIN_NODE, ACROSS_LEVELS] {
        let small = one_record_commit(&build(one), named);
        let large = one_record_commit(&build(10 * one), named);
        println!("{label} {one} keys: commit {small:?}");
        println!("{label} {} keys: commit {large:?}", 10 * one);
        let axis = format!("{label} {} vs {one} existing client keys", 10 * one);
        for (share, large, small) in [
            ("whole commit", large.all, small.all),
            ("symbols", large.symbols, small.symbols),
        ] {
            let calls = ratio(large.count, small.count);
            let bytes = ratio(large.bytes, small.bytes);
            println!("{axis}: {share} calls x{calls:.3}, bytes x{bytes:.3} (reported)");
        }
        assert_share_flat(
            large.beyond_storage_tree(),
            small.beyond_storage_tree(),
            &format!("{axis}: whole commit beyond the storage tree"),
        );
        if one == ACROSS_LEVELS {
            assert_share_flat(large.symbols, small.symbols, &format!("{axis}: symbols"));
        }
    }
}

#[test]
fn plain_one_record_commit_allocations_stay_flat_in_existing_client_keys() {
    assert_flat_in_existing_client_keys(plain, false, "plain");
}

#[test]
fn named_one_record_commit_allocations_stay_flat_in_existing_client_keys() {
    assert_flat_in_existing_client_keys(composed, true, "composed");
}
