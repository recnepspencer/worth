//! `forks_inheriting` over the real cutoff-indexed `fork_children` as a
//! conformance trace for the proved Lean model
//! (`proofs/lean/persistent_naming`, `ForkChildrenCost`).
//!
//! Forks are recorded with `record_fork_ancestry_metadata`, re-forks that
//! move a branch to another source included; a commit then asks which forks
//! inherit it. Each line records `fork_children` as Rust keeps it, the
//! committing branches, the commit and the forks `forks_inheriting` returned.
//! With `WORTH_PERSISTENT_NAMING_TRACE_DIR=<dir>` the lines are written to
//! `<dir>/fork_children.ndjson`, which `lake exe conformance` checks.
use std::fmt::Write as _;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::index::PersistentNameCommittedBatch;

use super::LineageState;

fn branch(n: u64) -> BranchId {
    BranchId(n.to_string())
}

fn number(branch: &BranchId) -> u64 {
    branch.0.parse().unwrap()
}

/// One commit's line over forks recorded as `(target, source, cutoff)`, and
/// the forks Rust found.
fn trace_commit(forks: &[(u64, u64, u64)], seeds: &[u64], commit: u64) -> (String, Vec<u64>) {
    let mut state = LineageState::empty();
    for &(target, source, cutoff) in forks {
        state.record_fork_ancestry_metadata(branch(target), branch(source), CommitId(cutoff));
    }
    let batches: Vec<_> = seeds
        .iter()
        .map(|&seed| {
            (
                branch(seed),
                PersistentNameCommittedBatch::new(CommitId(commit)),
            )
        })
        .collect();
    let mut found: Vec<u64> = state
        .forks_inheriting(&batches)
        .iter()
        .map(number)
        .collect();
    found.sort_unstable();
    let rows: Vec<String> = state
        .fork_children
        .iter()
        .map(|(parent, by_cutoff)| {
            let cutoffs: Vec<String> = by_cutoff
                .iter()
                .map(|(cutoff, children)| {
                    let children: Vec<u64> = children.iter().map(number).collect();
                    format!("[{},{children:?}]", cutoff.0)
                })
                .collect();
            format!("[{},[{}]]", number(parent), cutoffs.join(","))
        })
        .collect();
    let line = format!(
        r#"{{"kind":"fork_children","forks":[{}],"seeds":{seeds:?},"commit":{commit},"top":{},"found":{found:?}}}"#,
        rows.join(","),
        u64::MAX
    );
    (line, found)
}

/// Branch `0` commits at `100` over `older` forks taken before it, forks that
/// inherit it (`1` at the commit, `2` after it, `3` off `1`, `5` off `3`), and
/// forks that do not (`4` off `2` before the commit, `6` off `2` earlier).
fn inheriting_tree(older: u64) -> Vec<(u64, u64, u64)> {
    let mut forks: Vec<_> = (0..older).map(|i| (1000 + i, 0, 1 + i % 99)).collect();
    forks.extend([
        (1, 0, 100),
        (2, 0, 150),
        (3, 1, 120),
        (4, 2, 90),
        (5, 3, 130),
        (6, 2, 50),
    ]);
    forks
}

#[test]
fn forks_inheriting_finds_exactly_the_forks_reaching_the_commit_against_the_proved_model() {
    let mut trace = String::new();
    let mut found_over_older = Vec::new();
    for older in [0, 8, 64, 512] {
        let (line, found) = trace_commit(&inheriting_tree(older), &[0], 100);
        found_over_older.push(found);
        let _ = writeln!(trace, "{line}");
    }
    let mut moved = inheriting_tree(8);
    // `7` is re-forked from `4`, emptying `0`'s row at `110`; `8` from `5`.
    moved.extend([(7, 0, 110), (7, 4, 140), (8, 5, 100), (8, 1, 99)]);
    let cases: [(&[(u64, u64, u64)], &[u64], u64); 6] = [
        (&moved, &[0], 100),
        (&moved, &[4, 0], 100),
        (&moved, &[1, 0], 99),
        (&moved, &[2], 151),
        (&moved, &[9], 100),
        (&[], &[0], 100),
    ];
    for (forks, seeds, commit) in cases {
        let _ = writeln!(trace, "{}", trace_commit(forks, seeds, commit).0);
    }
    if let Ok(dir) = std::env::var("WORTH_PERSISTENT_NAMING_TRACE_DIR") {
        std::fs::write(format!("{dir}/fork_children.ndjson"), trace).unwrap();
    }
    for found in found_over_older {
        assert_eq!(found, [1, 2, 3, 5], "older forks never change the result");
    }
}
