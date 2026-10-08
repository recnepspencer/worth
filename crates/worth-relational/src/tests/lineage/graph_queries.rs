use crate::facade::history::BranchId;
use crate::facade::identity::{KindId, PartitionId};
use crate::facade::lineage::{
    LineageDecisionKind, LineageEventKind, LineageGraphDigestMode, LineageGraphRequest,
    LineageGraphTraversalBasis,
};
use crate::facade::transactions::{
    EntityMutationIntent, MutationIntent, ReplaceEntityIntent, WorkerIntentBatch,
};
use crate::tests::support::*;

// CONTRACT: lineage_graph_queries
// LANES: success, determinism

#[test]
fn lineage_graph_delete_emits_retire_event() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "retired");
    let entity = changed_entities(&created)[0];
    let deleted = delete_entity(&runtime, entity);
    let graph = runtime.lineage_access().graph(LineageGraphRequest {
        branch_id: BranchId("main".to_string()),
        traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
    });

    assert_eq!(
        graph.traversal_basis,
        LineageGraphTraversalBasis::FullBranchGraphMaterialization
    );
    assert!(graph
        .events
        .iter()
        .any(|event| event.commit.commit_id == deleted.commit.commit_id
            && event.kind == LineageEventKind::Retire));
}

#[test]
fn lineage_graph_replace_emits_replace_edge() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "source");
    let entity = changed_entities(&created)[0];

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: entity,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "replacement",
                    ),
                },
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let graph = runtime.lineage_access().graph(LineageGraphRequest {
        branch_id: BranchId("main".to_string()),
        traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
    });

    assert_eq!(
        graph.traversal_basis,
        LineageGraphTraversalBasis::FullBranchGraphMaterialization
    );
    assert!(graph.events.iter().any(|event| {
        event.commit.commit_id == outcome.commit.commit_id
            && event.kind == LineageEventKind::Replace
            && event.sources.len() == 1
            && event.targets.len() == 1
    }));
}

#[test]
fn lineage_graph_same_shape_replacements_do_not_cross_wire_targets() {
    let runtime = runtime_with_test_schema();
    let left = create_entity_outcome(&runtime, "left");
    let right = create_entity_outcome(&runtime, "right");
    let left_entity = changed_entities(&left)[0];
    let right_entity = changed_entities(&right)[0];

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace-many")
            .push(MutationIntent::Entity(EntityMutationIntent::Replace(
                ReplaceEntityIntent {
                    entity_id: left_entity,
                    replacement: crate::transactions::data::EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("replacement-left"),
                        fields: crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "replacement",
                        ),
                    },
                },
            )))
            .push(MutationIntent::Entity(EntityMutationIntent::Replace(
                ReplaceEntityIntent {
                    entity_id: right_entity,
                    replacement: crate::transactions::data::EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("replacement-right"),
                        fields: crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "replacement",
                        ),
                    },
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let graph = runtime.lineage_access().graph(LineageGraphRequest {
        branch_id: BranchId("main".to_string()),
        traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
    });
    let replace_events = graph
        .events
        .iter()
        .filter(|event| {
            event.commit.commit_id == outcome.commit.commit_id
                && event.kind == LineageEventKind::Replace
        })
        .collect::<Vec<_>>();

    assert_eq!(replace_events.len(), 2);
    assert_eq!(graph.metrics.event_count, graph.events.len());
    assert_eq!(graph.metrics.node_count, graph.nodes.len());
    assert_eq!(
        graph.digest_basis().digest_mode(),
        LineageGraphDigestMode::ExactDigestCanonicalOrder
    );
    assert_eq!(
        graph.digest_basis().canonical_event_ids(),
        graph
            .events
            .iter()
            .map(|event| event.event_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        graph.digest_basis().canonical_lineage_ids(),
        graph
            .nodes
            .iter()
            .map(|node| node.lineage_id)
            .collect::<Vec<_>>()
    );
    assert_ne!(replace_events[0].targets, replace_events[1].targets);
}

#[test]
fn lineage_graph_replace_commit_publishes_replace_decision_log_entry() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "source");
    let entity = changed_entities(&created)[0];

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: entity,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "replacement",
                    ),
                },
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(outcome.commit.commit_id)
        .unwrap();

    assert!(envelope
        .lineage_decision_log()
        .iter()
        .any(|decision| decision.kind == LineageDecisionKind::ReplaceAccepted));
}
