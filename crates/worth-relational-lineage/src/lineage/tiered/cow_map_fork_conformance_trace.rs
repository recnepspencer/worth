//! `CowMap` fork, structural-sharing diff (with real `Arc` pointer-identity
//! `NodeId` trees), and cold delta-chain conformance traces for the proved Lean
//! models (`proofs/lean/persistent_naming`, `CowMapForkCost` and `CowMapDeltaSeal`).
//!
//! With `WORTH_PERSISTENT_NAMING_TRACE_DIR=<dir>` lines are written to
//! `<dir>/cow_fork.ndjson` and `<dir>/cow_diff.ndjson`. When `lake` is
//! available (or `WORTH_REQUIRE_LEAN_CONFORMANCE=1`), the diff & cold-chain
//! test invokes `conformance` directly with positive and negative controls.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::process::Command;
use std::sync::Arc;

use super::{CowDiff, CowMap, Node};

#[path = "cow_map_cold_chain_conformance_trace.rs"]
mod cold_chain;

type Slot = Option<u64>;

fn nodes(map: &CowMap<u64, Slot>) -> BTreeSet<usize> {
    let mut found = BTreeSet::new();
    let mut stack: Vec<_> = map.root.iter().collect();
    while let Some(node) = stack.pop() {
        found.insert(Arc::as_ptr(node) as usize);
        stack.extend(node.left.iter().chain(node.right.iter()));
    }
    found
}

pub(super) fn fmt_opt(v: Option<u64>) -> String {
    v.map_or_else(|| "null".to_owned(), |n| n.to_string())
}

fn fmt_slot(s: Option<&Slot>) -> String {
    match s {
        None => "[]".to_owned(),
        Some(None) => "[null]".to_owned(),
        Some(Some(value)) => format!("[{value}]"),
    }
}

fn slots(map: &CowMap<u64, Slot>, probes: u64) -> String {
    let read: Vec<String> = (0..probes).map(|k| fmt_slot(map.get(&k))).collect();
    format!("[{}]", read.join(","))
}

pub(super) fn writes(writes: &[(u64, Slot)]) -> String {
    let rows: Vec<String> = writes
        .iter()
        .map(|(k, slot)| format!("[{k},{}]", fmt_opt(*slot)))
        .collect();
    format!("[{}]", rows.join(","))
}

fn fmt_diffs(diffs: &[CowDiff<'_, u64, Slot>]) -> String {
    let items: Vec<String> = diffs
        .iter()
        .map(|d| match d {
            CowDiff::Set(k, v, prev) => {
                format!(r#"["set",{k},{},{}]"#, fmt_opt(**v), fmt_slot(*prev))
            }
            CowDiff::Removed(k, prev) => format!(r#"["removed",{k},null,{}]"#, fmt_opt(**prev)),
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// Serializes an AVL `Node` tree assigning canonical `NodeId`s per distinct
/// `Arc::as_ptr` address (or fresh disjoint IDs when `force_fresh` is set).
fn fmt_id_tree(
    node: &Option<Arc<Node<u64, Slot>>>,
    ids: &mut BTreeMap<usize, u64>,
    next_id: &mut u64,
    force_fresh: bool,
) -> String {
    let Some(n) = node else {
        return "null".to_owned();
    };
    let id = if force_fresh {
        let fresh = *next_id;
        *next_id += 1;
        fresh
    } else {
        let addr = Arc::as_ptr(n) as usize;
        *ids.entry(addr).or_insert_with(|| {
            let assigned = *next_id;
            *next_id += 1;
            assigned
        })
    };
    let left = fmt_id_tree(&n.left, ids, next_id, force_fresh);
    let right = fmt_id_tree(&n.right, ids, next_id, force_fresh);
    format!(
        "[{id},{},{left},{},{},{right}]",
        n.height,
        n.key,
        fmt_opt(*n.val)
    )
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

fn trace_diff_case(
    size: u64,
    entries: Vec<(u64, Slot)>,
    child_writes: Vec<(u64, Slot)>,
    parent_writes: Vec<(u64, Slot)>,
    probes: u64,
) -> (String, Option<String>) {
    let mut parent = CowMap::new();
    for &(key, slot) in &entries {
        parent.insert(key, slot);
    }
    let mut child = parent.clone();
    let (id_diffs, id_stats) = child
        .diff_against_bounded_with_stats(&parent, usize::MAX)
        .unwrap();
    assert!(id_diffs.is_empty() && id_stats.visited == 0 && id_stats.ptr_checks == 1);
    for &(key, slot) in &child_writes {
        child.insert(key, slot);
    }
    for &(key, slot) in &parent_writes {
        parent.insert(key, slot);
    }
    let (diffs, stats) = child
        .diff_against_bounded_with_stats(&parent, usize::MAX)
        .unwrap();
    let mut ids = BTreeMap::new();
    let mut next_id = 1u64;
    let child_tree = fmt_id_tree(&child.root, &mut ids, &mut next_id, false);
    let parent_tree = fmt_id_tree(&parent.root, &mut ids, &mut next_id, false);
    if size >= 64 {
        assert!(
            (stats.visited as u64) < size,
            "diff_against must skip Arc::ptr_eq subtrees: visited={}, size={size}",
            stats.visited
        );
    }
    let limit = diffs.len();
    let (exact_diffs, _) = child
        .diff_against_bounded(&parent, limit)
        .expect("exact budget must succeed");
    assert_eq!(exact_diffs.len(), diffs.len());
    let tight_limit_refused = if limit > 0 {
        child.diff_against_bounded(&parent, limit - 1).is_none()
    } else {
        false
    };
    if limit > 0 {
        assert!(tight_limit_refused, "budget limit - 1 must abort");
    }
    let line = format!(
        r#"{{"kind":"cow_diff","size":{size},"entries":{},"child_writes":{},"parent_writes":{},"child_tree":{child_tree},"parent_tree":{parent_tree},"identical_visited":{},"visited":{},"ptr_checks":{},"push_steps":{},"limit":{limit},"tight_limit_refused":{tight_limit_refused},"diffs":{},"probes":{probes}}}"#,
        writes(&entries),
        writes(&child_writes),
        writes(&parent_writes),
        id_stats.visited,
        stats.visited,
        stats.ptr_checks,
        stats.push_steps,
        fmt_diffs(&diffs),
    );
    // Negative-control trace with pointer-skipping disabled: rebuild an unshared
    // clone with distinct `Arc`s so `visited` explodes to `>= size` and exceeds
    // the shared-subtree Lean step count!
    let unshared_neg = (size >= 64).then(|| {
        let mut unshared = CowMap::new();
        for &(key, slot) in &entries {
            unshared.insert(key, slot);
        }
        for &(key, slot) in &child_writes {
            unshared.insert(key, slot);
        }
        let (_, unshared_stats) = unshared
            .diff_against_bounded_with_stats(&parent, usize::MAX)
            .unwrap();
        assert!(
            unshared_stats.visited > stats.visited && (unshared_stats.visited as u64) >= size,
            "unshared negative control must exceed shared visited count: {} vs {} (size={size})",
            unshared_stats.visited,
            stats.visited
        );
        format!(
            r#"{{"kind":"cow_diff","size":{size},"entries":{},"child_writes":{},"parent_writes":{},"child_tree":{child_tree},"parent_tree":{parent_tree},"identical_visited":{},"visited":{},"ptr_checks":{},"push_steps":{},"limit":{limit},"tight_limit_refused":{tight_limit_refused},"diffs":{},"probes":{probes}}}"#,
            writes(&entries),
            writes(&child_writes),
            writes(&parent_writes),
            id_stats.visited,
            unshared_stats.visited,
            unshared_stats.ptr_checks,
            unshared_stats.push_steps,
            fmt_diffs(&diffs),
        )
    });
    (line, unshared_neg)
}

#[test]
fn cow_map_diff_and_cold_chain_refine_the_proved_lean_model() {
    cold_chain::assert_nondelta_tombstone_refuses();
    let mut trace = String::new();
    let mut unshared_neg_line: Option<String> = None;
    for size in [0u64, 1, 8, 64, 256] {
        let span = 2 * size + 1;
        let entries = (0..size + size / 4)
            .map(|i| ((i * 37) % span, (i % 5 != 0).then_some(i)))
            .collect();
        let child_w = vec![(0, Some(7)), (span / 2, None), (span, Some(9))];
        let parent_w = vec![(0, None), (1, Some(3)), (span + 1, Some(4))];
        let (line, neg) = trace_diff_case(size, entries, child_w, parent_w, span + 2);
        let _ = writeln!(trace, "{line}");
        if neg.is_some() {
            unshared_neg_line = neg;
        }
    }
    // Ascending/descending AVL rotation stress trace with divergent upper spines.
    let asc = (0..80u64).map(|k| (k * 2, Some(k + 100))).collect();
    let rot_c = (0..12u64).map(|k| (200 + k, Some(k))).collect();
    let rot_p = (0..12u64).rev().map(|k| (k * 2 + 1, Some(k * 7))).collect();
    let (rot_line, _) = trace_diff_case(80, asc, rot_c, rot_p, 220);
    let _ = writeln!(trace, "{rot_line}");
    let _ = writeln!(
        trace,
        "{}",
        cold_chain::trace_cold_chain_case(&[
            (
                "leaf",
                true,
                true,
                vec![(1, None), (3, Some(300)), (8, Some(800))]
            ),
            (
                "mid",
                true,
                true,
                vec![(2, Some(20)), (3, Some(30)), (5, None)]
            ),
            (
                "base",
                true,
                false,
                vec![(1, Some(1)), (2, Some(2)), (5, Some(5)), (7, Some(7))]
            ),
        ])
    );
    let _ = writeln!(
        trace,
        "{}",
        cold_chain::trace_cold_chain_case(&[
            ("leaf-stop", true, true, vec![(2, Some(22)), (4, None)]),
            ("mid-off", false, true, vec![(1, Some(11))]),
            ("base-hidden", true, false, vec![(1, Some(1)), (6, Some(6))]),
        ])
    );
    if let Ok(dir) = std::env::var("WORTH_PERSISTENT_NAMING_TRACE_DIR") {
        std::fs::write(format!("{dir}/cow_diff.ndjson"), &trace).unwrap();
    }
    if let Some(bin) = cold_chain::conformance_bin() {
        let tmp =
            std::env::temp_dir().join(format!("worth-cow-diff-{}.ndjson", std::process::id()));
        std::fs::write(&tmp, &trace).unwrap();
        let ok = Command::new(bin).arg(&tmp).output().unwrap();
        assert!(
            ok.status.success(),
            "Lean conformance failed:\n{}\n{}",
            String::from_utf8_lossy(&ok.stdout),
            String::from_utf8_lossy(&ok.stderr)
        );
        // Negative control 1: corrupt a cold-chain get result -> Lean must reject.
        let corrupted = trace.replacen(r#""gets":[null,null,20"#, r#""gets":[999,null,20"#, 1);
        assert_ne!(corrupted, trace);
        std::fs::write(&tmp, &corrupted).unwrap();
        let neg1 = Command::new(bin).arg(&tmp).output().unwrap();
        assert!(
            !neg1.status.success(),
            "Lean must reject corrupted cold-chain trace"
        );
        // Negative control 2: unshared visit count (disabled pointer-skipping) -> Lean must reject.
        std::fs::write(&tmp, unshared_neg_line.as_ref().unwrap()).unwrap();
        let neg2 = Command::new(bin).arg(&tmp).output().unwrap();
        let _ = std::fs::remove_file(&tmp);
        assert!(
            !neg2.status.success(),
            "Lean must reject unshared visit count exceeding shared-subtree step model"
        );
    }
}
