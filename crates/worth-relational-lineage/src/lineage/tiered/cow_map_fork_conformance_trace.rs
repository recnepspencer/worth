//! `CowMap` forks as a conformance trace for the proved Lean model
//! (`proofs/lean/persistent_naming`, `CowMapForkCost`).
//!
//! A map of `size` slots (values and tombstones, some keys written twice) is
//! cloned, the way `TieredMap::fork` forks a table; then each side writes its
//! own slots. Each line records the writes, the nodes the fork allocated and
//! whether it shares the source root, and the slots both sides read at every
//! probed key before and after the writes. With
//! `WORTH_PERSISTENT_NAMING_TRACE_DIR=<dir>` the lines are written to
//! `<dir>/cow_fork.ndjson`, which `lake exe conformance` checks.
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::Arc;

use super::CowMap;

type Slot = Option<u64>;

/// The addresses of every node of `map`'s tree.
fn nodes(map: &CowMap<u64, Slot>) -> BTreeSet<usize> {
    let mut found = BTreeSet::new();
    let mut stack: Vec<_> = map.root.iter().collect();
    while let Some(node) = stack.pop() {
        found.insert(Arc::as_ptr(node) as usize);
        stack.extend(node.left.iter().chain(node.right.iter()));
    }
    found
}

/// A slot as `[]` (absent), `[null]` (tombstone) or `[value]`.
fn slots(map: &CowMap<u64, Slot>, probes: u64) -> String {
    let read: Vec<String> = (0..probes)
        .map(|key| match map.get(&key) {
            None => "[]".to_owned(),
            Some(None) => "[null]".to_owned(),
            Some(Some(value)) => format!("[{value}]"),
        })
        .collect();
    format!("[{}]", read.join(","))
}

fn writes(writes: &[(u64, Slot)]) -> String {
    let rows: Vec<String> = writes
        .iter()
        .map(|(key, slot)| match slot {
            Some(value) => format!("[{key},{value}]"),
            None => format!("[{key},null]"),
        })
        .collect();
    format!("[{}]", rows.join(","))
}

fn trace_fork(size: u64) -> String {
    let span = 2 * size + 1;
    let entries: Vec<(u64, Slot)> = (0..size + size / 4)
        .map(|i| ((i * 37) % span, (i % 5 != 0).then_some(i)))
        .collect();
    let child_writes: Vec<(u64, Slot)> = [(0, Some(7)), (span / 2, None), (span, Some(9))].into();
    let parent_writes: Vec<(u64, Slot)> = [(0, None), (1, Some(3)), (span + 1, Some(4))].into();
    let mut parent = CowMap::new();
    for &(key, slot) in &entries {
        parent.insert(key, slot);
    }
    let source = nodes(&parent);
    let mut child = parent.clone();
    let fresh = nodes(&child).difference(&source).count();
    let shared = match (&parent.root, &child.root) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    };
    let probes = span + 2;
    let forked = slots(&child, probes);
    for &(key, slot) in &child_writes {
        child.insert(key, slot);
    }
    for &(key, slot) in &parent_writes {
        parent.insert(key, slot);
    }
    format!(
        r#"{{"kind":"cow_fork","entries":{},"fresh":{fresh},"shared":{shared},"probes":{probes},"forked":{forked},"child_writes":{},"parent_writes":{},"child":{},"parent":{}}}"#,
        writes(&entries),
        writes(&child_writes),
        writes(&parent_writes),
        slots(&child, probes),
        slots(&parent, probes),
    )
}

#[test]
fn cow_map_fork_shares_the_root_and_isolates_both_sides_against_the_proved_model() {
    let mut trace = String::new();
    for size in [0, 1, 8, 100, 1000] {
        let _ = writeln!(trace, "{}", trace_fork(size));
    }
    if let Ok(dir) = std::env::var("WORTH_PERSISTENT_NAMING_TRACE_DIR") {
        std::fs::write(format!("{dir}/cow_fork.ndjson"), trace).unwrap();
    }
}
