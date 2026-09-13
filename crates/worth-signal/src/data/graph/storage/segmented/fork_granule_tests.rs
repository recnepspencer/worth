use std::env;
use std::hash::{Hash, Hasher};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::SegmentedStore;
use crate::data::graph::storage::handles::DependencySetId;

static ITEM_CLONES: AtomicUsize = AtomicUsize::new(0);
static CLONE_PROBE: Mutex<()> = Mutex::new(());
const TEST_NAME: &str = "data::graph::storage::segmented::fork_granule_tests::repeated_fork_singleton_append_has_no_inherited_payload_slope";

#[derive(Debug, Eq, PartialEq)]
struct AllocatingClone(Box<u64>);

impl AllocatingClone {
    fn new(value: u64) -> Self {
        Self(Box::new(value))
    }
}

impl Clone for AllocatingClone {
    fn clone(&self) -> Self {
        ITEM_CLONES.fetch_add(1, Ordering::Relaxed);
        Self::new(*self.0)
    }
}

impl Hash for AllocatingClone {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

fn values(start: u64, len: usize) -> Vec<AllocatingClone> {
    (0..len)
        .map(|offset| AllocatingClone::new(start + offset as u64))
        .collect()
}

#[test]
fn repeated_fork_singleton_append_has_no_inherited_payload_slope() {
    const CHILD_PROCESS: &str = "WORTH_SIGNAL_SEGMENT_GRANULE_CHILD";
    // One changed vector page has 32 outer slots. These ceilings conservatively
    // allow one allocation and 512 bytes of bounded bookkeeping per slot.
    const MAX_CHANGED_PAGE_ALLOCATION_CALLS: usize = 32;
    const MAX_CHANGED_PAGE_ALLOCATED_BYTES: usize = 32 * 512;
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST_NAME)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD_PROCESS, "1")
            .output()
            .expect("isolated segment-granule probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "isolated segment-granule probe failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            stdout.contains(&format!("test {TEST_NAME} ... ok")),
            "isolated segment-granule probe selected no exact test\nstdout:\n{stdout}"
        );
        print!("{stdout}");
        eprint!("{stderr}");
        return;
    }

    let _probe = CLONE_PROBE
        .lock()
        .expect("clone probe lock remains healthy");
    let mut allocation_samples = Vec::new();
    for inherited_len in [64, 4_096, 65_536] {
        let mut original = SegmentedStore::<AllocatingClone, DependencySetId>::default();
        let base = original.insert_from_slice(&values(0, 1));
        let mut first_fork = original.fork_persistent();
        let inherited = first_fork.insert_from_slice(&values(10, inherited_len));
        ITEM_CLONES.store(0, Ordering::Relaxed);
        let mut repeated_fork = first_fork.fork_persistent();
        assert_eq!(ITEM_CLONES.load(Ordering::Relaxed), 0);
        assert!(first_fork.shares_storage_with(&repeated_fork));

        let singleton = AllocatingClone::new(u64::MAX);
        ITEM_CLONES.store(0, Ordering::Relaxed);
        let region = Region::new(&INSTRUMENTED_SYSTEM);
        let appended = repeated_fork.insert_from_slice(std::slice::from_ref(&singleton));
        let allocation = region.change();
        let item_clones = ITEM_CLONES.load(Ordering::Relaxed);
        allocation_samples.push((
            inherited_len,
            item_clones,
            allocation.allocations,
            allocation.bytes_allocated,
        ));

        assert_eq!(original.get(base), &[AllocatingClone::new(0)]);
        assert_eq!(first_fork.get(inherited).len(), inherited_len);
        assert_eq!(first_fork.live_segment_count(), 2);
        assert_eq!(repeated_fork.get(inherited).len(), inherited_len);
        assert_eq!(
            repeated_fork.get(appended),
            &[AllocatingClone::new(u64::MAX)]
        );
        assert_eq!(repeated_fork.live_segment_count(), 3);
    }

    eprintln!("repeated-fork singleton allocation samples: {allocation_samples:?}");
    // The complete insertion includes interner detachment. Hash layouts can
    // vary its bounded path-node cost, so every scale must satisfy the same
    // physical-granule ceiling; inherited item copying remains exactly zero.
    for (inherited_len, item_clones, calls, bytes) in allocation_samples {
        assert_eq!(
            item_clones, 1,
            "singleton append copied inherited payload at length {inherited_len}"
        );
        assert!(
            calls <= MAX_CHANGED_PAGE_ALLOCATION_CALLS,
            "whole insertion used {calls} allocations at inherited length {inherited_len}"
        );
        assert!(
            bytes <= MAX_CHANGED_PAGE_ALLOCATED_BYTES,
            "whole insertion allocated {bytes} bytes at inherited length {inherited_len}"
        );
    }
}

#[test]
fn cheap_clone_same_last_page_write_clones_only_the_new_payload() {
    let _probe = CLONE_PROBE
        .lock()
        .expect("clone probe lock remains healthy");
    let mut original = SegmentedStore::<AllocatingClone, DependencySetId>::default();
    let base = original.insert_from_slice(&values(0, 1));
    let mut first_fork = original.fork_persistent();
    let inherited = first_fork.insert_from_slice(&values(10, 64));
    let mut repeated_fork = first_fork.fork_persistent();
    let prior = repeated_fork.insert_from_slice(&values(100, 1));
    let sibling = repeated_fork.clone();
    assert!(repeated_fork.shares_storage_with(&sibling));

    let singleton = AllocatingClone::new(200);
    ITEM_CLONES.store(0, Ordering::Relaxed);
    let appended = repeated_fork.insert_from_slice(std::slice::from_ref(&singleton));

    assert_eq!(ITEM_CLONES.load(Ordering::Relaxed), 1);
    assert_eq!(original.get(base), &[AllocatingClone::new(0)]);
    assert_eq!(first_fork.get(inherited).len(), 64);
    assert_eq!(sibling.get(prior), &[AllocatingClone::new(100)]);
    assert_eq!(sibling.live_segment_count(), 3);
    assert_eq!(repeated_fork.get(appended), &[AllocatingClone::new(200)]);
    assert_eq!(repeated_fork.live_segment_count(), 4);

    ITEM_CLONES.store(0, Ordering::Relaxed);
    let operational = repeated_fork.operational_clone();
    assert_eq!(ITEM_CLONES.load(Ordering::Relaxed), 67);
    assert_eq!(operational, repeated_fork);
    assert!(!operational.shares_storage_with(&repeated_fork));
}

#[test]
fn repeated_fork_page_boundary_append_clones_only_the_new_payload() {
    const APPENDED_SEGMENTS_PER_PAGE: usize = 32;

    let _probe = CLONE_PROBE
        .lock()
        .expect("clone probe lock remains healthy");
    let mut original = SegmentedStore::<AllocatingClone, DependencySetId>::default();
    let base = original.insert_from_slice(&values(0, 1));
    let mut first_fork = original.fork_persistent();
    let mut inherited = Vec::new();
    for value in 0..APPENDED_SEGMENTS_PER_PAGE {
        inherited.push(first_fork.insert_from_slice(&values(1_000 + value as u64, 1)));
    }
    let mut repeated_fork = first_fork.fork_persistent();
    assert!(first_fork.shares_storage_with(&repeated_fork));

    let singleton = AllocatingClone::new(2_000);
    ITEM_CLONES.store(0, Ordering::Relaxed);
    let appended = repeated_fork.insert_from_slice(std::slice::from_ref(&singleton));

    assert_eq!(ITEM_CLONES.load(Ordering::Relaxed), 1);
    assert_eq!(original.get(base), &[AllocatingClone::new(0)]);
    assert_eq!(
        first_fork.live_segment_count(),
        APPENDED_SEGMENTS_PER_PAGE + 1
    );
    for (offset, id) in inherited.into_iter().enumerate() {
        assert_eq!(
            first_fork.get(id),
            &[AllocatingClone::new(1_000 + offset as u64)]
        );
        assert_eq!(repeated_fork.get(id), first_fork.get(id));
    }
    assert_eq!(repeated_fork.get(appended), &[AllocatingClone::new(2_000)]);
    assert_eq!(
        repeated_fork.live_segment_count(),
        APPENDED_SEGMENTS_PER_PAGE + 2
    );
}
