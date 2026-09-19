use super::*;
use crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis;
use crate::data::aspect::{Aspect, AspectVersion};
use crate::data::dependency::{DependencySnapshot, DependencySnapshotEntry};
use crate::data::graph::storage::evaluation_partition::{
    ConditionalEvaluationDraft, SignalEvaluationPartition,
};
use crate::data::output::PartitionSubscription;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation as Reservation,
};
use crate::runtime_policy::SignalConditionalEvaluationBudget;
use std::sync::Arc;

fn fixture(replacement: bool, scope: &str) -> (SignalGraph, ApplyCommitPacket, DependencySnapshot) {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let source = graph.create_node();
    let entry = |version, detail: &str| DependencySnapshotEntry {
        source,
        aspect: Aspect::new(0),
        cached_version: version,
        scope: Some(PartitionSubscription::partition_and_detail(scope, detail)),
    };
    let previous = DependencySnapshot::from_ordered_unique([entry(1, "old")]);
    graph.set_dep_snapshot(node, previous.clone()).unwrap();
    let next = DependencySnapshot::from_ordered_unique([entry(
        2,
        if replacement { "new" } else { "old" },
    )]);
    let (_, id) = graph.node_dependency_ids(node).unwrap();
    let shape = graph.dependency_snapshot_shape_handle(id);
    let (update, delta) =
        CommittedSnapshotUpdate::between(node, id, shape, &previous, next.clone());
    assert_eq!(
        matches!(update, CommittedSnapshotUpdate::Replace(_)),
        replacement
    );
    let mut effect = super::super::super::tests::test_effect_with_labels(Vec::new());
    effect.operational.node = node;
    effect.operational.dependency_snapshot_update = update;
    effect.operational.snapshot_delta = delta;
    effect.operational.aspect_version = AspectVersion::zero();
    let apply = graph
        .build_apply_commit_packet(
            effect,
            OutputEquivalencePolicy::ExactAspectVersion,
            false,
            &mut EvaluationWork::Ordinary,
        )
        .unwrap();
    (graph, apply, next)
}

fn with_retained_fixture<R>(
    replacement: bool,
    maximum_bytes: u64,
    run: impl FnOnce(
        &mut SignalGraph,
        &ApplyCommitPacket,
        &DependencySnapshot,
        &mut SignalEvaluationPartition,
        &Arc<SignalConditionalRetentionLedger>,
    ) -> R,
) -> R {
    let (mut graph, apply, expected) = fixture(replacement, &"scope".repeat(64));
    let ledger = SignalConditionalRetentionLedger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 8,
            maximum_retained_bytes: maximum_bytes,
            maximum_attempt_visits: 50_000_000,
        },
        crate::runtime_policy::SignalRuntimePolicy::development().conditional_temporal_budget,
    );
    let basis =
        SignalRetainedExecutionBasis::capture(&mut graph, &ledger, &mut Work::new(50_000_000))
            .unwrap();
    let mut slot = basis
        .new_evaluation_partition(&mut Work::new(50_000_000))
        .unwrap();
    let output = run(&mut graph, &apply, &expected, &mut slot, &ledger);
    drop(slot);
    drop(basis);
    drop(graph);
    assert_eq!(ledger.usage(), (0, 0));
    output
}

fn same_custody(
    left: &Option<Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>>,
    right: &Option<Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

#[test]
fn materialized_snapshot_admission_is_shared_and_precedes_graph_movement() {
    for replacement in [false, true] {
        let (mut graph, apply, expected) = fixture(replacement, &"scope".repeat(1024));
        let node = apply.effect.operational.node;
        let before = graph.get_dep_snapshot(node).unwrap().clone();
        let before_id = graph.node_dependency_ids(node).unwrap();
        let mut measured = Work::new(usize::MAX);
        graph
            .materialize_effect_snapshot(&apply, &mut EvaluationWork::Conditional(&mut measured))
            .unwrap()
            .unwrap();
        let cost = measured.visits();
        assert!(cost >= 5 * 1024);
        for available in [cost - 1, cost] {
            let mut work = Work::new(cost + 13);
            work.reserve_visits(cost + 13 - available).unwrap();
            let result = graph
                .materialize_effect_snapshot(&apply, &mut EvaluationWork::Conditional(&mut work));
            if available == cost {
                let prepared = result.unwrap().unwrap();
                assert_eq!(
                    prepared.delta.changed_entry_count,
                    if replacement { 2 } else { 1 }
                );
            } else {
                assert!(
                    matches!(result, Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }) if maximum_visits == cost + 13)
                );
            }
            assert_eq!(graph.get_dep_snapshot(node).unwrap(), &before);
            assert_eq!(graph.node_dependency_ids(node).unwrap(), before_id);
        }
        // Exercise the actual packet builder and publisher, not only the narrow
        // materialization seam used above to isolate its allowance.
        let packet = graph
            .prepare_output_commit_packet(
                apply,
                &mut crate::data::comparator::DefaultComparatorPolicyResolver::default(),
                &mut EvaluationWork::Ordinary,
            )
            .unwrap();
        graph.publish_output_commit_packet(packet);
        assert_eq!(graph.get_dep_snapshot(node).unwrap(), &expected);
        assert_eq!(before.entries()[0].cached_version, 1);
    }
}

#[test]
fn snapshot_materialization_charges_scope_payload_and_skips_deferred_commit() {
    for replacement in [false, true] {
        let (mut short_graph, short, _) = fixture(replacement, "x");
        let mut measured = Work::new(10_000_000);
        short_graph
            .materialize_effect_snapshot(&short, &mut EvaluationWork::Conditional(&mut measured))
            .unwrap();
        let (mut graph, mut long, _) = fixture(replacement, &"x".repeat(4096));
        assert!(matches!(
            graph.materialize_effect_snapshot(
                &long,
                &mut EvaluationWork::Conditional(&mut Work::new(measured.visits()))
            ),
            Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
        ));
        long.defer_snapshot_commit = true;
        assert!(graph
            .materialize_effect_snapshot(&long, &mut EvaluationWork::Conditional(&mut Work::new(0)))
            .unwrap()
            .is_none());
    }
}

#[test]
fn retained_snapshot_publication_has_atomic_capacity_and_exact_custody() {
    const MAXIMUM_BYTES: u64 = 128 * 1024 * 1024;
    for replacement in [false, true] {
        with_retained_fixture(
            replacement,
            MAXIMUM_BYTES,
            |graph, apply, expected, slot, ledger| {
                slot.execute(graph, |selected| {
                    let baseline = ledger.usage();
                    let free_payload =
                        MAXIMUM_BYTES - baseline.1 - std::mem::size_of::<Reservation>() as u64;
                    let admits = |selected: &mut SignalGraph, pressure_bytes: u64| {
                        let pressure = ledger
                            .reserve(0, Charge::capacity::<u8>(pressure_bytes as usize).unwrap())
                            .unwrap();
                        let result = selected.materialize_effect_snapshot(
                            apply,
                            &mut EvaluationWork::Conditional(&mut Work::new(usize::MAX)),
                        );
                        let admitted = result.is_ok();
                        drop(result);
                        drop(pressure);
                        assert_eq!(ledger.usage(), baseline);
                        admitted
                    };
                    assert!(admits(selected, 0));
                    assert!(!admits(selected, free_payload));
                    let (mut admitted_pressure, mut denied_pressure) = (0, free_payload);
                    while admitted_pressure + 1 < denied_pressure {
                        let middle = admitted_pressure + (denied_pressure - admitted_pressure) / 2;
                        if admits(selected, middle) {
                            admitted_pressure = middle;
                        } else {
                            denied_pressure = middle;
                        }
                    }
                    assert_eq!(denied_pressure, admitted_pressure + 1);

                    let node = apply.effect.operational.node;
                    let before_snapshot = selected.get_dep_snapshot(node).unwrap().clone();
                    let before_ids = selected.node_dependency_ids(node).unwrap();
                    let before_snapshots = selected.topology.dependency_snapshots.clone();
                    let before_shapes = selected.topology.dependency_snapshot_shapes.clone();
                    let before_custody = selected
                        .topology
                        .dependency_snapshot_storage_custody
                        .clone();
                    let denied_pressure = ledger
                        .reserve(0, Charge::capacity::<u8>(denied_pressure as usize).unwrap())
                        .unwrap();
                    assert_eq!(
                        selected
                            .materialize_effect_snapshot(
                                apply,
                                &mut EvaluationWork::Conditional(&mut Work::new(usize::MAX)),
                            )
                            .unwrap_err(),
                        SignalError::EvaluationStorageCapacityExhausted
                    );
                    assert_eq!(selected.get_dep_snapshot(node).unwrap(), &before_snapshot);
                    assert_eq!(selected.node_dependency_ids(node).unwrap(), before_ids);
                    assert_eq!(selected.topology.dependency_snapshots, before_snapshots);
                    assert_eq!(selected.topology.dependency_snapshot_shapes, before_shapes);
                    assert!(same_custody(
                        &selected.topology.dependency_snapshot_storage_custody,
                        &before_custody
                    ));
                    drop(denied_pressure);
                    assert_eq!(ledger.usage(), baseline);

                    let admitted_pressure = ledger
                        .reserve(
                            0,
                            Charge::capacity::<u8>(admitted_pressure as usize).unwrap(),
                        )
                        .unwrap();
                    let publication = selected
                        .materialize_effect_snapshot(
                            apply,
                            &mut EvaluationWork::Conditional(&mut Work::new(usize::MAX)),
                        )
                        .unwrap()
                        .unwrap();
                    let published_id = publication.node_snapshot_id().unwrap();
                    publication.publish(selected);
                    drop(admitted_pressure);
                    assert_eq!(
                        selected.topology.dependency_snapshots.get(published_id),
                        expected
                    );
                    assert_eq!(selected.node_dependency_ids(node).unwrap(), before_ids);
                    assert!(selected
                        .topology
                        .dependency_snapshot_storage_custody
                        .is_some());
                    let plateau = ledger.usage();
                    let repeated = selected
                        .materialize_effect_snapshot(
                            apply,
                            &mut EvaluationWork::Conditional(&mut Work::new(usize::MAX)),
                        )
                        .unwrap()
                        .unwrap();
                    repeated.publish(selected);
                    assert_eq!(ledger.usage(), plateau);
                    assert_eq!(
                        selected.topology.dependency_snapshots.get(published_id),
                        expected
                    );
                })
                .unwrap();
            },
        );
    }
}

#[test]
fn rejected_draft_keeps_retained_snapshot_custody_until_release() {
    with_retained_fixture(true, u64::MAX, |graph, apply, _, slot, ledger| {
        ConditionalEvaluationDraft::begin(slot, &mut Work::new(50_000_000))
            .unwrap()
            .install();
        let baseline = ledger.usage();
        let draft = ConditionalEvaluationDraft::begin(slot, &mut Work::new(50_000_000)).unwrap();
        draft
            .partition
            .execute(graph, |selected| {
                let publication = selected
                    .materialize_effect_snapshot(
                        apply,
                        &mut EvaluationWork::Conditional(&mut Work::new(usize::MAX)),
                    )
                    .unwrap()
                    .unwrap();
                publication.publish(selected);
            })
            .unwrap();
        let rejected = draft.reject();
        assert!(ledger.usage().1 > baseline.1);
        drop(rejected);
        assert_eq!(ledger.usage(), baseline);
    });
}
