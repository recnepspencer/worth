//! The one stack-attributing tracker behind the binary's global allocator,
//! shared by the Layer-3 scaling courts (RULES §4). While a measured run
//! executes, every allocation on every thread is counted, and attributed to
//! an owner family when any frame of its stack passes through that owner.
//! No production counter is involved and no owner path can opt out. One
//! tracker per binary: courts that share a process share this one.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use tracking_allocator::{AllocationGroupId, AllocationRegistry, AllocationTracker};

/// Persistent naming: the lineage owner (minting, mint order, posture
/// validation, finalization, the name index) and the lineage birth codec.
const NAMING: &[&str] = &[
    "worth_relational_lineage::",
    "::lineage::data::",
    "group_finalization::",
    "lineage_preparation::",
];
/// The derived name index: ingestion, the tail-only journal, carrier deltas
/// and member tenures.
const NAME_INDEX: &[&str] = &["::lineage::index::"];
/// The Relational symbol dictionary: its text carrier and interner, the
/// runtime table, candidate reservations and publication. Owner paths, not
/// `symbols::data::symbol::Symbol`, which storage column types also name.
const SYMBOLS: &[&str] = &[
    "::symbols::data::string_interner::",
    "::symbols::data::stored_text::",
    "::symbols::data::commit_texts::",
    "::symbol_table::",
];
/// Relational storage's persistent record columns and maps (`ColumnNode` and
/// `MapNode` path copies), whose depth grows with the record count.
const STORAGE_TREE: &[&str] = &["::storage::substrate::"];
/// Copies of a symbol text: a `StoredSymbolText` clone that allocates.
const TEXT_COPIES: &[&str] = &["StoredSymbolText as core::clone::Clone>::clone"];
const FAMILIES: [&[&str]; 5] = [NAMING, NAME_INDEX, SYMBOLS, STORAGE_TREE, TEXT_COPIES];

static MEASURING: AtomicBool = AtomicBool::new(false);
static COUNTS: [AtomicUsize; 2 * (1 + FAMILIES.len())] =
    [const { AtomicUsize::new(0) }; 2 * (1 + FAMILIES.len())];

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
        for (family, owners) in FAMILIES.iter().enumerate() {
            if owners.iter().any(|owner| stack.contains(owner)) {
                COUNTS[2 + 2 * family].fetch_add(1, Relaxed);
                COUNTS[3 + 2 * family].fetch_add(bytes, Relaxed);
            }
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

/// Allocation calls and bytes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Share {
    pub(crate) count: usize,
    pub(crate) bytes: usize,
}

/// One measured run's allocations: in all, and each owner family's share.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Allocations {
    pub(crate) all: Share,
    pub(crate) naming: Share,
    pub(crate) name_index: Share,
    pub(crate) symbols: Share,
    pub(crate) storage_tree: Share,
    pub(crate) text_copies: Share,
}

pub(crate) fn measure<R>(run: impl FnOnce() -> R) -> Allocations {
    measure_with(run).0
}

pub(crate) fn measure_with<R>(run: impl FnOnce() -> R) -> (Allocations, R) {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        AllocationRegistry::set_global_tracker(Attribution).expect("one tracker per binary");
        AllocationRegistry::enable_tracking();
    });
    COUNTS.iter().for_each(|count| count.store(0, Relaxed));
    MEASURING.store(true, Relaxed);
    let result = run();
    MEASURING.store(false, Relaxed);
    let share = |at: usize| Share {
        count: COUNTS[at].load(Relaxed),
        bytes: COUNTS[at + 1].load(Relaxed),
    };
    let allocations = Allocations {
        all: share(0),
        naming: share(2),
        name_index: share(4),
        symbols: share(6),
        storage_tree: share(8),
        text_copies: share(10),
    };
    (allocations, result)
}

impl Allocations {
    /// Every allocation outside the storage tree family.
    pub(crate) fn beyond_storage_tree(&self) -> Share {
        Share {
            count: self.all.count - self.storage_tree.count,
            bytes: self.all.bytes - self.storage_tree.bytes,
        }
    }
}

/// `large` over `small`; a zero `small` share counts as one.
pub(crate) fn ratio(large: usize, small: usize) -> f64 {
    large as f64 / small.max(1) as f64
}
