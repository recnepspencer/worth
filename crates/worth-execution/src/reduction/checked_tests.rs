use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::{plan::priority, ReductionDenial, ReductionPlan, ReductionRunStop, ReductionTree};

fn id(value: u64) -> PartitionIdentity {
    PartitionIdentity::new(value)
}

fn plan(ids: &[u64]) -> ReductionPlan {
    ReductionPlan::try_from_sorted_unique(ids.iter().copied().map(id).collect()).unwrap()
}

fn from_plan<T, F>(
    plan: ReductionPlan,
    values: Vec<T>,
    identity: T,
    combine: F,
) -> Result<(ReductionTree<T, F>, super::ReductionMetrics), ReductionDenial>
where
    T: Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T,
{
    let entries = plan.identities().iter().copied().zip(values).collect();
    ReductionTree::try_from_declared(plan, entries, identity, combine)
}

#[test]
fn stop_between_combines_keeps_retained_root_and_reports_entered_work() {
    let (mut tree, _) = from_plan(plan(&[1]), vec![3_u64], 0, |a, b| a + b).unwrap();
    let mut calls = 0;
    let stopped = tree
        .update_checked(id(1), 7, 8, || {
            calls += 1;
            if calls == 2 {
                Err("stop")
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert_eq!(stopped.reason, ReductionRunStop::Hook("stop"));
    assert_eq!(stopped.metrics.combine_calls, 1);
    assert_eq!(stopped.metrics.recombined_nodes, 0);
    assert_eq!(stopped.metrics.charged_work, 1);
    assert_eq!(*tree.result(), 3);
    assert_eq!(tree.partition_count(), 1);
    assert_eq!(tree.update(id(1), 7).unwrap().combine_calls, 2);
    assert_eq!(*tree.result(), 7);
}

#[test]
fn reducer_panic_during_each_edit_keeps_old_tree_usable() {
    let panic_now = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&panic_now);
    let reducer = move |a: &u64, b: &u64| {
        if flag.load(Ordering::Relaxed) {
            panic!("injected combine panic");
        }
        a + b
    };
    let (mut tree, _) = from_plan(plan(&[1, 2]), vec![3, 5], 0, reducer).unwrap();
    panic_now.store(true, Ordering::Relaxed);
    assert_eq!(
        tree.update_checked(id(1), 7, 8, || Ok::<_, ()>(()))
            .unwrap_err()
            .reason,
        ReductionRunStop::Panic
    );
    assert_eq!(
        tree.insert_checked(id(3), 9, 8, || Ok::<_, ()>(()))
            .unwrap_err()
            .reason,
        ReductionRunStop::Panic
    );
    let leaf = if priority(id(1)) < priority(id(2)) {
        id(2)
    } else {
        id(1)
    };
    assert_eq!(
        tree.delete_checked(leaf, 8, || Ok::<_, ()>(()))
            .unwrap_err()
            .reason,
        ReductionRunStop::Panic
    );
    assert_eq!(*tree.result(), 8);
    assert_eq!(tree.partition_count(), 2);
    assert!(matches!(
        tree.update(id(1), 7),
        Err(ReductionDenial::ReducerPanic)
    ));
    assert_eq!(*tree.result(), 8);
    panic_now.store(false, Ordering::Relaxed);
    tree.update(id(1), 7).unwrap();
    tree.insert(id(3), 9).unwrap();
    tree.delete(id(2)).unwrap();
    assert_eq!(*tree.result(), 16);
}

#[derive(Clone)]
struct BadEncoding {
    value: u64,
    malformed: bool,
}

impl ChargedBytes for BadEncoding {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl CanonicalBits for BadEncoding {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        if self.malformed {
            visit(&self.value.to_le_bytes()[..4])
        } else {
            visit(&self.value.to_le_bytes())
        }
    }
}

#[test]
fn malformed_encoding_and_capacity_rejection_do_not_publish() {
    let reducer = |a: &BadEncoding, b: &BadEncoding| BadEncoding {
        value: a.value + b.value,
        malformed: a.malformed || b.malformed,
    };
    let (mut tree, _) = from_plan(
        plan(&[1]),
        vec![BadEncoding {
            value: 3,
            malformed: false,
        }],
        BadEncoding {
            value: 0,
            malformed: false,
        },
        reducer,
    )
    .unwrap();
    let invalid = tree
        .update_checked(
            id(1),
            BadEncoding {
                value: 9,
                malformed: true,
            },
            16,
            || Ok::<_, ()>(()),
        )
        .unwrap_err();
    assert_eq!(
        invalid.reason,
        ReductionRunStop::Denial(ReductionDenial::InvalidCanonicalEncoding)
    );
    assert_eq!(invalid.metrics.combine_calls, 0);
    assert_eq!(tree.result().value, 3);
    let capacity = tree
        .update_checked(
            id(1),
            BadEncoding {
                value: 9,
                malformed: false,
            },
            8,
            || Ok::<_, ()>(()),
        )
        .unwrap_err();
    assert_eq!(capacity.reason, ReductionRunStop::ResultCapacityExceeded);
    assert_eq!(tree.result().value, 3);
}

#[test]
fn malformed_combine_output_is_rejected_after_one_charged_call() {
    let malformed = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&malformed);
    let reducer = move |a: &BadEncoding, b: &BadEncoding| BadEncoding {
        value: a.value + b.value,
        malformed: flag.load(Ordering::Relaxed),
    };
    let (mut tree, _) = from_plan(
        plan(&[1]),
        vec![BadEncoding {
            value: 3,
            malformed: false,
        }],
        BadEncoding {
            value: 0,
            malformed: false,
        },
        reducer,
    )
    .unwrap();
    malformed.store(true, Ordering::Relaxed);
    let failed = tree
        .update_checked(
            id(1),
            BadEncoding {
                value: 9,
                malformed: false,
            },
            16,
            || Ok::<_, ()>(()),
        )
        .unwrap_err();
    assert_eq!(
        failed.reason,
        ReductionRunStop::Denial(ReductionDenial::InvalidCanonicalEncoding)
    );
    assert_eq!(failed.metrics.combine_calls, 1);
    assert_eq!(tree.result().value, 3);
}

#[test]
fn checked_update_distinguishes_signed_zero() {
    let (mut tree, _) = from_plan(plan(&[1]), vec![0.0_f64], 1.0, |a, b| a * b).unwrap();
    let metrics = tree
        .update_checked(id(1), -0.0, 8, || Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(metrics.combine_calls, 2);
    assert_eq!(tree.result().to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn checked_incremental_edits_match_fresh_canonical_float_tree() {
    let mut entries = vec![(id(1), 1.0e20_f64), (id(2), -1.0e20), (id(3), 3.0)];
    let (mut tree, _) = from_plan(
        plan(&[1, 2, 3]),
        entries.iter().map(|entry| entry.1).collect(),
        0.0,
        |a, b| a + b,
    )
    .unwrap();
    tree.update_checked(id(2), -1.0e20 + 16384.0, 8, || Ok::<_, ()>(()))
        .unwrap();
    entries[1].1 = -1.0e20 + 16384.0;
    tree.insert_checked(id(4), 0.25, 8, || Ok::<_, ()>(()))
        .unwrap();
    entries.push((id(4), 0.25));
    tree.delete_checked(id(1), 8, || Ok::<_, ()>(())).unwrap();
    entries.remove(0);
    let ids = entries.iter().map(|entry| entry.0).collect();
    let (fresh, _) = ReductionTree::try_from_declared_checked(
        ReductionPlan::try_from_sorted_unique(ids).unwrap(),
        entries,
        0.0,
        |a, b| a + b,
        8,
        || Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(tree.result().to_bits(), fresh.result().to_bits());
}

#[test]
fn full_tree_span_tracks_dependency_depth() {
    let ids: Vec<_> = (0..128).collect();
    let (_, metrics) = ReductionTree::from_plan_checked(
        plan(&ids),
        vec![1_u64; ids.len()],
        0,
        |a, b| a + b,
        8,
        || Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(metrics.combine_calls, 2 * ids.len() as u64);
    assert_eq!(
        metrics.charged_work,
        metrics.combine_calls + metrics.structural_visits
    );
    assert!(metrics.structural_visits <= 7 * ids.len() as u64);
    assert!(metrics.charged_span < metrics.charged_work);
}

#[test]
fn staged_child_completion_order_preserves_canonical_bits_and_metrics() {
    let ids: Vec<_> = (1..=64).collect();
    let values: Vec<_> = ids.iter().map(|id| (*id as f64).sin()).collect();
    let combine = |left: &f64, right: &f64| left + right;
    let (oracle, oracle_metrics) =
        ReductionTree::from_plan_checked(plan(&ids), values.clone(), 0.0, combine, 8, || {
            Ok::<_, ()>(())
        })
        .unwrap();
    let (mut shape, shape_metrics) =
        ReductionTree::prepare_shape_checked(plan(&ids), values, &0.0, &combine, 8, || {
            Ok::<_, ()>(())
        })
        .unwrap();
    let frontier = shape.frontier(4);
    let mut children = frontier.tasks.clone();
    assert_eq!(children.len(), 4);
    children.reverse();
    let mut results = children
        .iter()
        .map(|spec| {
            ReductionTree::evaluate_subtree_checked(&shape, *spec, &0.0, &combine, 8, || {
                Ok::<_, ()>(())
            })
            .unwrap()
        })
        .collect::<Vec<_>>();
    results.reverse();
    let (staged, metrics) = ReductionTree::settle_frontier_checked(
        shape,
        0.0,
        combine,
        shape_metrics,
        frontier,
        results,
        None,
        8,
        |_| Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(staged.result().to_bits(), oracle.result().to_bits());
    assert_eq!(metrics, oracle_metrics);
}
