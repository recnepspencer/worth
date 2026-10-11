//! Constructed tree faults carry a nonzero canonical prefix on every placement.
use super::super::worker_axis::placed;
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedTreeRebuildCause as Rebuild, WorthQueryPartitionedTreeRun as TreeRun,
};
use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;

const PARTITIONS: usize = 32;

/// Independent postorder: root last, or a deep leaf in the first quarter.
fn fault_positions() -> [u32; 2] {
    use worth_foundational::facade::PartitionIdentity as Id;
    fn postorder(keys: &[(Id, u32)], depth: usize, out: &mut Vec<(u32, usize, bool)>) {
        let Some((root, _)) = keys
            .iter()
            .enumerate()
            .min_by_key(|(_, (key, _))| tree_work::priority(*key))
        else {
            return;
        };
        postorder(&keys[..root], depth + 1, out);
        postorder(&keys[root + 1..], depth + 1, out);
        out.push((keys[root].1, depth, keys.len() == 1));
    }
    let mut keys: Vec<_> = (0..PARTITIONS as u32)
        .map(|region| (tree_work::identity(region), region))
        .collect();
    keys.sort_by_key(|(key, _)| *key);
    let mut order = Vec::new();
    postorder(&keys, 0, &mut order);
    // Leave at least one completed node before the fault. Other ready leaves
    // can run past this early canonical boundary at wider placements.
    let early = order
        .iter()
        .skip(1)
        .take(PARTITIONS / 4 - 1)
        .filter(|(_, depth, leaf)| *leaf && *depth >= 3)
        .max_by_key(|(_, depth, _)| *depth)
        .expect("the declared shape has an early deep leaf")
        .0;
    [order.last().unwrap().0, early]
}

fn stopped(
    placement: Placement,
    retained: bool,
    fault: u32,
    serial: Option<&[TreeRun]>,
) -> OracleRun {
    let application = installation::install_variant::<false, TOTALS_WORK, 1, 4>(
        None,
        Default::default(),
        |graph| {
            facts::seed_set(graph, "even", -0.0);
            for number in 0..PARTITIONS {
                seed_entry(
                    graph,
                    &["even"],
                    number,
                    RegionEntry {
                        id: number as u64,
                        region: number as u32,
                        value: if !retained && number as u32 == fault {
                            tree_count::FAULT_LEAF
                        } else {
                            1.0
                        },
                        work: 1,
                        fault: None,
                    },
                );
            }
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    placed(placement, || {
        if retained {
            let (_, first) = demand(&request, &application);
            assert!(first[0].outcome.is_ok());
            edit(
                &request,
                &application,
                EntryEdit::new(
                    "even",
                    fault as u64,
                    super::super::entry_edit::EntryFact::Value,
                    tree_count::FAULT_LEAF.to_bits(),
                ),
                1,
            );
        }
        let (_, mut runs) = demand_reconciled(&request, &application, serial);
        assert_eq!(runs.len(), 1);
        runs.pop().unwrap()
    })
}

#[test]
fn stopped_tree_builds_match_serial_and_stopped_edits_are_exact_at_every_width() {
    let _guard = checkpoint_recovery_test_guard();
    for fault in fault_positions() {
        for retained in [false, true] {
            let serial = stopped(Placement::Serial, retained, fault, None);
            assert!(matches!(
                serial.outcome,
                Err(WorthQueryPartitionedComputationDenial::ReducerPanicked)
            ));
            let [tree] = serial.tree_runs.as_slice() else {
                panic!("one stopped tree report")
            };
            assert!(tree.metrics().combine_calls > 0);
            assert!(tree.metrics().recombined_nodes > 0);
            if retained {
                assert!(matches!(
                    tree,
                    TreeRun::Rebuilt(Rebuild::ReducerPanicked, _)
                ));
            } else {
                assert!(matches!(tree, TreeRun::Full(_, _)));
            }
            for workers in [1, 2, 4] {
                let run = stopped(
                    Placement::Leased(NonZeroUsize::new(workers).unwrap()),
                    retained,
                    fault,
                    Some(&serial.tree_runs),
                );
                assert_eq!(run.outcome, serial.outcome);
                assert_eq!(run.tree_runs, serial.tree_runs);
            }
        }
    }
}
