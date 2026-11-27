//! Layer-3 persistent-naming complexity court (RULES §4). While a measured
//! commit runs through the real Relational runtime (a structural-identity
//! scheme, Anchor group partitions and a forked branch composed), the
//! binary's global allocator attributes every allocation on every thread
//! by stack: it is naming work when the stack passes through the lineage
//! owner (`worth_relational_lineage`: minting, mint order, posture
//! validation, finalization, the name index) or the lineage birth codec. No
//! production counter is involved and no naming path can opt out. Nextest
//! runs each court in its own process, so no other court allocates here.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use tracking_allocator::{AllocationGroupId, AllocationRegistry, AllocationTracker};

mod world;

use world::World;
use worth_relational::facade::lineage::PersistentNameIdentity;

static MEASURING: AtomicBool = AtomicBool::new(false);
static COUNTS: [AtomicUsize; 6] = [const { AtomicUsize::new(0) }; 6];
/// The derived name index: ingestion, the tail-only journal, carrier deltas
/// and member tenures (hot path #4).
const INDEX_OWNER: &str = "::lineage::index::";
const NAMING_OWNERS: [&str; 4] = [
    "worth_relational_lineage::",
    "::lineage::data::",
    "group_finalization::",
    "lineage_preparation::",
];

struct Attribution;
impl AllocationTracker for Attribution {
    fn allocated(&self, _: usize, bytes: usize, _: usize, _: AllocationGroupId) {
        if !MEASURING.load(Relaxed) {
            return;
        }
        COUNTS[0].fetch_add(1, Relaxed);
        COUNTS[1].fetch_add(bytes, Relaxed);
        // The capture's own allocations are not re-entered by the tracker.
        let stack = std::backtrace::Backtrace::force_capture().to_string();
        if NAMING_OWNERS.iter().any(|owner| stack.contains(owner)) {
            COUNTS[2].fetch_add(1, Relaxed);
            COUNTS[3].fetch_add(bytes, Relaxed);
        }
        if stack.contains(INDEX_OWNER) {
            COUNTS[4].fetch_add(1, Relaxed);
            COUNTS[5].fetch_add(bytes, Relaxed);
        }
    }
    fn deallocated(
        &self,
        _: usize,
        _: usize,
        _: usize,
        _: AllocationGroupId,
        _: AllocationGroupId,
    ) {
    }
}

/// Allocation calls and bytes of one commit: in all, naming's share, and
/// the derived name index's share of that.
#[derive(Clone, Copy, Debug)]
struct Cost {
    count: usize,
    bytes: usize,
    naming_count: usize,
    naming_bytes: usize,
    index_count: usize,
    index_bytes: usize,
}

fn measure(run: impl FnOnce()) -> Cost {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        AllocationRegistry::set_global_tracker(Attribution).expect("one tracker per binary");
        AllocationRegistry::enable_tracking();
    });
    COUNTS.iter().for_each(|count| count.store(0, Relaxed));
    MEASURING.store(true, Relaxed);
    run();
    MEASURING.store(false, Relaxed);
    let [count, bytes, naming_count, naming_bytes, index_count, index_bytes] =
        COUNTS.each_ref().map(|c| c.load(Relaxed));
    assert!(
        naming_count > 0,
        "naming work is attributed: stacks carry symbols"
    );
    Cost {
        count,
        bytes,
        naming_count,
        naming_bytes,
        index_count,
        index_bytes,
    }
}

fn ratio(large: usize, small: usize) -> f64 {
    large as f64 / small as f64
}

/// Naming's allocation calls and bytes stay within `bound` of `small`. The
/// whole commit's growth is reported, not bounded: it includes storage and
/// symbol work outside naming.
fn assert_naming_flat(large: Cost, small: Cost, bound: f64, axis: &str) {
    let calls = ratio(large.naming_count, small.naming_count);
    let bytes = ratio(large.naming_bytes, small.naming_bytes);
    let commit_calls = ratio(large.count, small.count);
    let commit_bytes = ratio(large.bytes, small.bytes);
    println!("{axis}: naming calls x{calls:.3}, bytes x{bytes:.3}");
    println!("{axis}: whole commit calls x{commit_calls:.3}, bytes x{commit_bytes:.3}");
    assert!(
        calls <= bound && bytes <= bound,
        "{axis}: {large:?} vs {small:?}"
    );
}

fn keys(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("{prefix}.{index}"))
        .collect()
}

/// Splits `body` `depth` times in a chain, every piece told apart by the
/// retired cutter's 32-byte name and its step. Returns the deepest piece's
/// encoded birth and the cost of the deepest split's commit.
fn split_chain(depth: u64) -> (usize, Cost) {
    let world = World::new();
    world.authored(&world.main(), &["cutter".into(), "body".into()]);
    let cutter = World::authored_name("cutter");
    let mut body = World::authored_name("body");
    world.retire(&world.main(), cutter);
    for step in 1..depth {
        let pieces = [
            (format!("body.{step}"), 2 * step),
            (format!("sliver.{step}"), 2 * step + 1),
        ];
        let basis = world.main();
        let (intents, plan, names) = world.split(&basis, body, cutter, &pieces);
        world.commit(&basis, intents, plan);
        body = names[0];
    }
    let pieces = [
        ("body.last".to_owned(), 2 * depth),
        ("sliver.last".to_owned(), 2 * depth + 1),
    ];
    let basis = world.main();
    let (intents, plan, names) = world.split(&basis, body, cutter, &pieces);
    let deepest = measure(|| world.commit(&basis, intents, plan));
    (world.birth_bytes(&world.main(), names[0]), deepest)
}

/// One commit that splits a body by a retired cutter, over `background`
/// authored names grouped in eights, a sixteenth of them retired, and a
/// forked branch.
fn commit_over(background: usize) -> Cost {
    let world = World::new();
    let mut names = Vec::new();
    for batch in 0..background / 64 {
        let batch = keys(&format!("background.{batch}"), 64);
        world.authored(&world.main(), &batch);
        names.extend(batch.iter().map(|key| World::authored_name(key)));
    }
    world.group(&world.main(), &names, 8);
    for name in names.iter().step_by(16) {
        world.retire(&world.main(), *name);
    }
    world.fork("side");
    world.authored(&world.main(), &["cutter".into(), "body".into()]);
    let cutter = World::authored_name("cutter");
    world.retire(&world.main(), cutter);
    let basis = world.main();
    let pieces = [("piece.a".to_owned(), 1), ("piece.b".to_owned(), 2)];
    let (intents, plan, _) = world.split(&basis, World::authored_name("body"), cutter, &pieces);
    measure(|| world.commit(&basis, intents, plan))
}

#[test]
fn naming_allocations_stay_flat_in_split_depth_and_background_names() {
    let (shallow_birth, shallow) = split_chain(2);
    let (deep_birth, deep) = split_chain(64);
    println!("split depth 2: birth {shallow_birth} B, commit {shallow:?}");
    println!("split depth 64: birth {deep_birth} B, commit {deep:?}");
    let birth = ratio(deep_birth, shallow_birth);
    println!("split depth 64 vs 2: birth bytes x{birth:.3}");
    assert!(birth <= 1.05, "a bounded birth at any split depth");
    assert_naming_flat(deep, shallow, 1.1, "split depth 64 vs 2");

    let one = commit_over(64);
    let ten = commit_over(640);
    println!("background 1x (64 names): commit {one:?}");
    println!("background 10x (640 names): commit {ten:?}");
    assert_naming_flat(ten, one, 1.1, "background 10x vs 1x");
}

/// Authors `count` member names in commits of 64; returns them in order.
fn authored_members(world: &World, prefix: &str, count: usize) -> Vec<PersistentNameIdentity> {
    let mut names = Vec::new();
    for batch in 0..count.div_ceil(64) {
        let batch = keys(&format!("{prefix}.{batch}"), 64.min(count - batch * 64));
        world.authored(&world.main(), &batch);
        names.extend(batch.iter().map(|key| World::authored_name(key)));
    }
    names
}

/// An Anchor group of `members` led by its first, beside retired names and
/// a forked branch.
fn group_of(members: usize) -> (World, Vec<PersistentNameIdentity>) {
    let world = World::new();
    let names = authored_members(&world, "member", members + 2);
    world.group(&world.main(), &names[..members], members);
    world.retire(&world.main(), names[members]);
    world.fork("side");
    (world, names)
}

/// One commit detaching the last member of an Anchor group of `members`
/// into its own group: the group keeps its name and loses one member.
fn detach_one_of(members: usize) -> Cost {
    let (world, names) = group_of(members);
    let (intents, plan) = World::partition(vec![vec![names[members - 1]]]);
    let basis = world.main();
    measure(|| world.commit(&basis, intents, plan))
}

/// One commit retiring the last member of an Anchor group of `members`.
fn retire_one_of(members: usize) -> Cost {
    let (world, names) = group_of(members);
    measure(|| world.retire(&world.main(), names[members - 1]))
}

/// One commit detaching a member from an Anchor group of eight, or joining
/// it back, after `history` such commits.
fn regroup_after(history: usize) -> Cost {
    let (world, names) = group_of(8);
    let toggle = |step: usize| match step % 2 {
        0 => World::partition(vec![vec![names[7]]]),
        _ => World::partition(vec![names[..8].to_vec()]),
    };
    for step in 0..history {
        let (intents, plan) = toggle(step);
        world.commit(&world.main(), intents, plan);
    }
    let (intents, plan) = toggle(history);
    let basis = world.main();
    measure(|| world.commit(&basis, intents, plan))
}

/// The derived index's allocation calls stay within `bound` of `small`;
/// its bytes may grow only logarithmically in the group size, the persistent
/// carrier set's path copy gaining a level (`members_ratio` = log2 large /
/// log2 small group size), never with a copy of the set.
fn assert_index_flat(large: Cost, small: Cost, bound: f64, members_ratio: f64, axis: &str) {
    let calls = ratio(large.index_count, small.index_count.max(1));
    let bytes = ratio(large.index_bytes, small.index_bytes.max(1));
    println!("{axis}: derived index calls x{calls:.3}, bytes x{bytes:.3} (log bound x{members_ratio:.3})");
    assert!(
        calls <= bound && bytes <= members_ratio,
        "{axis}: {large:?} vs {small:?}"
    );
}

#[test]
fn naming_allocations_stay_flat_in_group_size_and_membership_history() {
    let short = regroup_after(32);
    let long = regroup_after(320);
    println!("membership history 32: commit {short:?}");
    println!("membership history 320: commit {long:?}");
    assert_naming_flat(long, short, 1.1, "membership history 320 vs 32");

    // A one-member touch of a 64- or 640-member group. The derived index
    // (carrier deltas, tail-only journal, tenures) is flat; the canonical
    // group transition still names the touched group's complete next-state
    // membership, so naming as a whole is reported, not bounded, on this
    // axis (the canonical delta frame is the recorded follow-on).
    for (label, touch) in [
        ("detach", detach_one_of as fn(usize) -> Cost),
        ("retire", retire_one_of),
    ] {
        let small = touch(64);
        let large = touch(640);
        println!("{label} one of 64: commit {small:?}");
        println!("{label} one of 640: commit {large:?}");
        let calls = ratio(large.naming_count, small.naming_count);
        let bytes = ratio(large.naming_bytes, small.naming_bytes);
        println!("{label} 640 vs 64: naming calls x{calls:.3}, bytes x{bytes:.3} (reported)");
        let log_ratio = 640f64.log2() / 64f64.log2();
        let axis = format!("{label} one of 640 vs 64");
        assert_index_flat(large, small, 1.1, log_ratio, &axis);
    }
}
