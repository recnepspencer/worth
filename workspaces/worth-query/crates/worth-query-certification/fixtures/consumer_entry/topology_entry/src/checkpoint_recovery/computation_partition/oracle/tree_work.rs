//! Independent reducer entries and completed pairs bound physical tree work;
//! the runtime separately reports canonical tree posture and accounting.

use super::*;
use worth_foundational::facade::PartitionIdentity as Id;
use worth_query_host::facade::application_contribution::WorthQueryPartitionedTreeRun as TreeRun;

#[path = "tree_shape.rs"]
mod shape;

pub(super) enum TreeStop<'a> {
    Exact,
    BeforeTree,
    ParallelBuild { serial: &'a [TreeRun] },
}

pub(super) fn stop_kind<'a>(run: &OracleRun, serial: Option<&'a [TreeRun]>) -> TreeStop<'a> {
    use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;
    if run.outcome.is_err() && run.tree_runs.is_empty() {
        TreeStop::BeforeTree
    } else if run.outcome.is_err()
        && matches!(run.tree_runs.as_slice(), [TreeRun::Full(_, _)])
        && matches!(run.placement, Placement::Leased(n) | Placement::Certified { workers: n, .. } if n.get() > 1)
    {
        TreeStop::ParallelBuild {
            serial: serial.expect("a stopped parallel build requires its serial witness"),
        }
    } else {
        TreeStop::Exact
    }
}

pub(super) fn assert_counted(run: &OracleRun) {
    assert_reconciled(run, TreeStop::Exact);
}

/// Physical completed pairs, excluding the observer's two later certification builds.
pub(super) fn completed_nodes(run: &OracleRun) -> u128 {
    (run.tree_nodes as u128)
        .checked_sub(2 * certification_leaves(run))
        .unwrap()
}
pub(super) fn reducer_calls(run: &OracleRun) -> u128 {
    (run.combines as u128)
        .checked_sub(4 * certification_leaves(run))
        .unwrap()
}
fn certification_leaves(run: &OracleRun) -> u128 {
    use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;
    if run
        .runs
        .iter()
        .any(|run| matches!(run, WorthQueryPartitionedComputationRunForTest::Full(_)))
        && matches!(run.placement, Placement::Certified { .. })
    {
        assert_eq!(run.calls.kernels, 3 * run.calls.gathers);
        run.calls.gathers as u128
    } else {
        0
    }
}
pub(super) fn assert_reconciled(run: &OracleRun, stop: TreeStop<'_>) {
    let calls = reducer_calls(run);
    let nodes = completed_nodes(run);
    assert!(
        calls >= 2 * nodes,
        "completed pairs require actual reducer entries"
    );
    if run.outcome.is_ok() {
        assert_eq!(calls, 2 * nodes);
    }
    if let TreeStop::ParallelBuild { serial } = stop {
        assert_eq!(
            run.tree_runs, serial,
            "canonical stopped work equals the serial prefix"
        );
    }
    if matches!(stop, TreeStop::BeforeTree) {
        assert_eq!((calls, nodes), (0, 0), "map-stage stops enter no reducer");
    }
}

pub(super) fn priority(key: Id) -> (u64, Id) {
    shape::priority(key)
}

pub(super) fn identity(region: u32) -> Id {
    use sha2::{Digest, Sha256};
    // Canonical contract: domain and identity have big-endian u64 lengths;
    // newtype tag 14 precedes its LEB128 name length and name; unsigned tag 3
    // precedes the integer's minimal unsigned LEB128 encoding.
    let mut bytes = Vec::new();
    for text in [
        "worth-query.computation-partition-key.v1",
        "checkpoint-region-key",
    ] {
        bytes.extend_from_slice(&(text.len() as u64).to_be_bytes());
        bytes.extend_from_slice(text.as_bytes());
    }
    bytes.extend_from_slice(&[14, 9]);
    bytes.extend_from_slice(b"RegionKey");
    bytes.push(3);
    let mut remaining = region;
    loop {
        let group = (remaining & 127) as u8;
        remaining >>= 7;
        bytes.push(group | if remaining == 0 { 0 } else { 128 });
        if remaining == 0 {
            break;
        }
    }
    let digest = Sha256::digest(bytes);
    Id::new(u64::from_be_bytes(digest[..8].try_into().unwrap()))
}

pub(super) fn build_charge(keys: &[Id]) -> u64 {
    shape::Shape::build_charge(&shape::Shape::from_sorted(keys))
}

pub(super) fn keys(size: usize) -> Vec<Id> {
    let mut keys: Vec<_> = (0..size)
        .map(|n| identity(u32::try_from(n).unwrap()))
        .collect();
    keys.sort();
    keys
}

/// Root-to-leaf identities for 6.11's immutable node-sharing proof.
pub(super) fn update_path(keys: &[Id], key: Id) -> Vec<Id> {
    shape::Shape::update_path(&shape::Shape::from_sorted(keys), key)
}

pub(super) fn update_bound(keys: &[Id], key: Id) -> u64 {
    u64::try_from(update_path(keys, key).len()).unwrap()
}

pub(super) fn insert_bound(keys: &[Id], key: Id) -> u64 {
    let tree = shape::Shape::from_sorted(keys);
    shape::Shape::depth(&tree, key) + shape::Shape::rotations(&tree, key) + 1
}

pub(super) fn delete_bound(keys: &[Id], key: Id) -> u64 {
    shape::Shape::deletion_depth(&shape::Shape::from_sorted(keys), key)
}

pub(super) fn assert_edited(run: &OracleRun, bound: u64) {
    assert_counted(run);
    let [TreeRun::Edited(_)] = run.tree_runs.as_slice() else {
        panic!(
            "partition reuse has its own tree-edit dimension: {:?}",
            run.tree_runs
        )
    };
    assert!(
        completed_nodes(run) <= u128::from(bound),
        "recombined {} nodes, pre-edit shape bound {bound}",
        completed_nodes(run)
    );
    assert_eq!(reducer_calls(run), 2 * completed_nodes(run));
}

#[test]
fn recomputed_partition_with_identical_bits_reports_zero_tree_work() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(|graph| {
        facts::seed_set(graph, "even", -0.0);
        for place in 0..32 {
            seed_entry(
                graph,
                &["even"],
                place,
                RegionEntry {
                    id: u64::try_from(place).unwrap(),
                    region: u32::try_from(place).unwrap(),
                    value: 1.0,
                    work: 1,
                    fault: None,
                },
            );
        }
    });
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (_, first) = demand(&request, &application);
    assert_counted(&first[0]);
    // An ignored probe fault moves a gathered fact while preserving the
    // leaf's canonical bits. Owner calls are independent of tree work.
    edit(
        &request,
        &application,
        EntryEdit::new("even", 7, super::super::entry_edit::EntryFact::Fault, 3),
        1,
    );
    let (_, next) = demand(&request, &application);
    assert_eq!(
        next[0].calls,
        OwnerCalls {
            plans: 0,
            keys: 0,
            gathers: 1,
            kernels: 1
        }
    );
    assert_eq!(
        next[0].runs,
        [WorthQueryPartitionedComputationRunForTest::Incremental]
    );
    assert_edited(&next[0], 0);
    assert_eq!(
        next[0].outcome.as_ref().unwrap().0,
        first[0].outcome.as_ref().unwrap().0
    );
}
