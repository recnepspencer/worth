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

static MEASURING: AtomicBool = AtomicBool::new(false);
static COUNTS: [AtomicUsize; 4] = [const { AtomicUsize::new(0) }; 4];
const NAMING_OWNERS: [&str; 2] = ["worth_relational_lineage::", "::lineage::data::"];

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

/// Allocation calls and bytes of one commit: in all, and naming's share.
#[derive(Clone, Copy, Debug)]
struct Cost {
    count: usize,
    bytes: usize,
    naming_count: usize,
    naming_bytes: usize,
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
    let [count, bytes, naming_count, naming_bytes] = COUNTS.each_ref().map(|c| c.load(Relaxed));
    assert!(
        naming_count > 0,
        "naming work is attributed: stacks carry symbols"
    );
    Cost {
        count,
        bytes,
        naming_count,
        naming_bytes,
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
