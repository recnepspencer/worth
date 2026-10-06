//! Requested-object peaks for the selected node draft and root owner.
use std::collections::BTreeSet;

use super::{OperationalNodePayload, RetainedNodePayload, SelectedNodeDraft, SelectedNodeRole};
use crate::data::graph::runtime::graph::{RetainedNodeEditPreparation, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::node::{NodeColdData, NodeState};
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger,
};
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};
use crate::tests::performance_support::peak_allocation::measure;

const SELECTED: usize = 1_024;
const UNSELECTED: usize = 64;

fn fixture(retained: bool) -> (SignalGraph, Vec<NodeId>) {
    let mut graph = SignalGraph::new();
    let ids = (0..SELECTED + UNSELECTED)
        .map(|index| {
            let node = graph.create_node();
            let label = if index < SELECTED {
                "selected cold".repeat(64)
            } else {
                "unselected cold".repeat(4_096)
            };
            graph.arena.cold.replace_discard(
                index,
                Some(Box::new(NodeColdData {
                    causality: Some(crate::data::trace::CausalityMetadata {
                        kind: label,
                        fields: Default::default(),
                    }),
                    ..Default::default()
                })),
            );
            node
        })
        .collect::<Vec<_>>();
    if retained {
        fork_node_roots(&mut graph);
        graph.arena.retained_node_ledger = Some(SignalConditionalRetentionLedger::new(
            SignalConditionalEvaluationBudget {
                maximum_retained_slots: 8,
                maximum_retained_bytes: 2 * 1024 * 1024 * 1024,
                maximum_attempt_visits: usize::MAX,
            },
            SignalRuntimePolicy::development().conditional_temporal_budget,
        ));
    }
    (graph, ids)
}

fn fork_node_roots(graph: &mut SignalGraph) {
    let mut work = Work::new(usize::MAX);
    graph.arena.hot.prepare_retained_charge(&mut work).unwrap();
    graph.arena.warm.prepare_retained_charge(&mut work).unwrap();
    graph.arena.cold.prepare_retained_charge(&mut work).unwrap();
    graph.arena.hot = graph.arena.hot.fork_persistent();
    graph.arena.warm = graph.arena.warm.fork_persistent();
    graph.arena.cold = graph.arena.cold.fork_persistent();
}

fn roles(ids: &[NodeId]) -> Vec<(usize, SelectedNodeRole)> {
    ids.iter()
        .take(SELECTED)
        .enumerate()
        .map(|(index, _)| {
            let role = if index == 0 || index == SELECTED / 2 {
                SelectedNodeRole::ProducerFull
            } else {
                SelectedNodeRole::ConsumerOperational
            };
            (index, role)
        })
        .collect()
}

fn predispatch_owner_bytes(graph: &SignalGraph, ids: &[NodeId], retained: bool) -> u64 {
    let producer = ids[0];
    let second_producer = ids[SELECTED / 2];
    let prior_producers = BTreeSet::from([producer]);
    let mut work = Work::new(usize::MAX);
    let mut budget = SignalPreparationBudget::new(u64::MAX);
    let root = graph
        .epoch_selected_node_root_growth_bound(
            &BTreeSet::new(),
            &ids[..SELECTED],
            &prior_producers,
            second_producer,
            &mut work,
            &mut budget,
        )
        .unwrap();
    let drafts = ids[..SELECTED]
        .iter()
        .enumerate()
        .map(|(index, &node)| {
            let charge = if index == 0 || index == SELECTED / 2 {
                graph.epoch_producer_draft_clone_charge(node, &mut work)
            } else {
                graph.epoch_consumer_clone_charge(node, &mut work)
            };
            charge.unwrap().bytes()
        })
        .sum::<u64>();
    let structure = graph
        .epoch_node_edit_structure_bound(SELECTED, SELECTED, 2)
        .unwrap();
    assert_eq!(retained, graph.arena.retained_node_ledger.is_some());
    root + drafts + structure
}

#[test]
fn retained_mixed_role_node_roots_fit_the_actual_predispatch_owner_bound() {
    let (graph, ids) = fixture(true);
    let selections = roles(&ids);
    let owner_bound = predispatch_owner_bytes(&graph, &ids, true);
    let unselected_cold = graph.arena.cold[SELECTED].as_deref().unwrap() as *const NodeColdData;
    let ledger = graph.arena.retained_node_ledger.as_ref().unwrap();
    let (prepared, peak) = measure(|| {
        graph
            .arena
            .prepare_selected_epoch_nodes(
                ledger,
                &selections,
                Charge::capacity::<u8>(2 * 1024 * 1024 * 1024).unwrap(),
                &mut Work::new(usize::MAX),
                |drafts, _| {
                    for draft in drafts {
                        match draft {
                            SelectedNodeDraft::ProducerFull(payload) => {
                                payload.hot.state = NodeState::Clean;
                            }
                            SelectedNodeDraft::ConsumerOperational(payload) => {
                                payload.hot.state = NodeState::MaybeStale;
                            }
                        }
                    }
                },
            )
            .unwrap()
    });
    assert!(matches!(prepared, RetainedNodeEditPreparation::Ready(_)));
    let peak = peak.expect("selected root allocations were tracked") as u64;
    assert!(
        peak > 0 && peak <= owner_bound,
        "peak {peak}, owner {owner_bound}"
    );
    assert_eq!(
        graph.arena.cold[SELECTED].as_deref().unwrap() as *const NodeColdData,
        unselected_cold,
    );
}

#[test]
fn ordinary_mixed_role_move_only_replacement_fits_the_predispatch_owner_bound() {
    ordinary_peak(false);
    ordinary_peak(true);
}

fn ordinary_peak(shared: bool) {
    let (mut graph, ids) = fixture(false);
    if shared {
        fork_node_roots(&mut graph);
    }
    let selections = roles(&ids);
    let owner_bound = predispatch_owner_bytes(&graph, &ids, false);
    if !shared {
        assert_eq!(
            graph
                .epoch_selected_node_root_growth_bound(
                    &BTreeSet::new(),
                    &ids[..SELECTED],
                    &BTreeSet::from([ids[0]]),
                    ids[SELECTED / 2],
                    &mut Work::new(usize::MAX),
                    &mut SignalPreparationBudget::new(u64::MAX),
                )
                .unwrap(),
            0
        );
    }
    let unselected_cold = graph.arena.cold[SELECTED].as_deref().unwrap() as *const NodeColdData;
    let ((), peak) = measure(|| {
        let drafts = selections
            .iter()
            .map(|&(index, role)| {
                let hot = graph.arena.hot[index].as_ref().unwrap().clone();
                let draft = match role {
                    SelectedNodeRole::ProducerFull => {
                        SelectedNodeDraft::ProducerFull(RetainedNodePayload {
                            hot,
                            warm: graph.arena.warm[index].clone(),
                            cold: graph.arena.cold[index].clone(),
                        })
                    }
                    SelectedNodeRole::ConsumerOperational => {
                        SelectedNodeDraft::ConsumerOperational(OperationalNodePayload {
                            hot,
                            warm: graph.arena.warm[index].operational_consumer_draft(),
                        })
                    }
                };
                (index, draft)
            })
            .collect::<Vec<_>>();
        for (index, draft) in drafts {
            match draft {
                SelectedNodeDraft::ProducerFull(payload) => {
                    graph.arena.hot.replace_discard(index, Some(payload.hot));
                    graph.arena.warm.replace_discard(index, payload.warm);
                    graph.arena.cold.replace_discard(index, payload.cold);
                }
                SelectedNodeDraft::ConsumerOperational(payload) => {
                    graph.arena.hot.replace_discard(index, Some(payload.hot));
                    graph.arena.warm.replace_discard(index, payload.warm);
                }
            }
        }
    });
    let peak = peak.expect("ordinary selected allocations were tracked") as u64;
    assert!(
        peak > 0 && peak <= owner_bound,
        "peak {peak}, owner {owner_bound}"
    );
    assert_eq!(
        graph.arena.cold[SELECTED].as_deref().unwrap() as *const NodeColdData,
        unselected_cold,
    );
}
