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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "retired");
    let entity = changed_entities(&created)[0];
    let deleted = delete_entity(execution, &runtime, entity);
    let graph = runtime
        .lineage_access()
        .graph(
            execution,
            LineageGraphRequest {
                branch_id: BranchId("main".to_string()),
                traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
            },
        )
        .unwrap();

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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
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
    let outcome = txn.commit(execution, &runtime).unwrap();
    let graph = runtime
        .lineage_access()
        .graph(
            execution,
            LineageGraphRequest {
                branch_id: BranchId("main".to_string()),
                traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
            },
        )
        .unwrap();

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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let left = create_entity_outcome(execution, &runtime, "left");
    let right = create_entity_outcome(execution, &runtime, "right");
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
    let outcome = txn.commit(execution, &runtime).unwrap();
    let graph = runtime
        .lineage_access()
        .graph(
            execution,
            LineageGraphRequest {
                branch_id: BranchId("main".to_string()),
                traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
            },
        )
        .unwrap();
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
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
    let outcome = txn.commit(execution, &runtime).unwrap();
    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(execution, outcome.commit.commit_id)
        .unwrap()
        .unwrap();

    assert!(envelope
        .lineage_decision_log()
        .iter()
        .any(|decision| decision.kind == LineageDecisionKind::ReplaceAccepted));
}

#[test]
fn lineage_postures_finalize_through_runtime_preparation_authority_and_refuse_corrupt_claims() {
    use std::collections::BTreeMap;

    use crate::facade::identity::VersionId;
    use crate::facade::lineage::{
        LineageCommitPostureInput, LineageMergeKeeper, LineageNameDenial, LineagePostureClaim,
        LineagePostureKind, LineageRecordTarget, LineageSplitChild, LineageSubject,
        PersistentNameHolder,
    };
    use crate::facade::transactions::RecordRef;

    use super::posture_fixtures::{
        cad_scheme_registry, push_relation, receipt_at, retire_entity, retire_relation, seam_birth,
        seed_two_faces_and_parallel_seams, staged_entity_lineage,
    };

    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let prep = runtime.preparation_runtime_snapshot();
    let authority = prep.lineage_preparation_authority();
    let registry = cad_scheme_registry();
    let (mut staged, f1, f2, s1, s2, c1_changed, c1_input) = seed_two_faces_and_parallel_seams();
    let c1 = authority
        .ensure_lineage_for_commit_with_postures(
            execution,
            &mut staged,
            &receipt_at(1, 1),
            &[],
            &c1_changed,
            &c1_input,
        )
        .expect("runtime posture birth commit succeeds");

    let c1_ev = c1.artifact().event_batch().events();
    let (f1_lin, f2_lin) = (
        staged_entity_lineage(&staged, f1).unwrap(),
        staged_entity_lineage(&staged, f2).unwrap(),
    );
    let (s1_lin, s2_lin) = (c1.relation_lineages()[&s1], c1.relation_lineages()[&s2]);
    let name_of = |lin| {
        c1_ev
            .iter()
            .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [lin])
            .unwrap()
            .name_birth()
            .unwrap()
            .identity()
    };
    let (f1_name, f2_name, s1_name, s2_name) = (
        name_of(f1_lin),
        name_of(f2_lin),
        name_of(s1_lin),
        name_of(s2_lin),
    );
    assert_ne!(s1_name, s2_name);
    let live_holders = BTreeMap::from([
        (f1_name, PersistentNameHolder::Entity(f1)),
        (f2_name, PersistentNameHolder::Entity(f2)),
        (s1_name, PersistentNameHolder::Relation(s1)),
        (s2_name, PersistentNameHolder::Relation(s2)),
    ]);
    let live_lineages = BTreeMap::from([
        (PersistentNameHolder::Entity(f1), f1_lin),
        (PersistentNameHolder::Entity(f2), f2_lin),
        (PersistentNameHolder::Relation(s1), s1_lin),
        (PersistentNameHolder::Relation(s2), s2_lin),
    ]);

    let before = prep.lineage_identity.frontiers();
    let err = authority
        .ensure_lineage_for_commit_with_postures(
            execution,
            &mut staged,
            &receipt_at(2, 2),
            &[],
            &[],
            &LineageCommitPostureInput {
                scheme_registry: registry.clone(),
                live_holders: live_holders.clone(),
                live_lineages: live_lineages.clone(),
                postures: vec![LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Entity(f1),
                    name: f1_name,
                }],
                ..Default::default()
            },
        )
        .err()
        .expect("unretired Retire must refuse");
    assert!(matches!(
        err.lineage_name_denial_ref(),
        Some(LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Retire,
            ..
        })
    ));
    assert_eq!(prep.lineage_identity.frontiers(), before);

    retire_entity(&mut staged, f2, VersionId(2));
    retire_relation(&mut staged, s1, VersionId(2));
    let s1_a = push_relation(&mut staged, VersionId(2), f1, f1);
    let s1_b = push_relation(&mut staged, VersionId(2), f1, f1);
    let c2_changed = [
        RecordRef::Entity(f2),
        RecordRef::Relation(s1),
        RecordRef::Relation(s1_a),
        RecordRef::Relation(s1_b),
    ];
    let c2_input = LineageCommitPostureInput {
        scheme_registry: registry,
        live_holders,
        live_lineages,
        postures: vec![
            LineagePostureClaim::Merge {
                parents: vec![
                    (LineageRecordTarget::Entity(f1), f1_name),
                    (LineageRecordTarget::Entity(f2), f2_name),
                ],
                target: LineageRecordTarget::Entity(f1),
                keeper: LineageMergeKeeper::KeepParent(f1_name),
            },
            LineagePostureClaim::Split {
                parent_target: LineageRecordTarget::Relation(s1),
                parent_name: s1_name,
                children: vec![
                    LineageSplitChild {
                        target: LineageRecordTarget::Relation(s1_a),
                        birth: seam_birth("occ.rt.2", "seam_piece", vec![s1_name], "a", s1_a),
                    },
                    LineageSplitChild {
                        target: LineageRecordTarget::Relation(s1_b),
                        birth: seam_birth("occ.rt.2", "seam_piece", vec![s1_name], "b", s1_b),
                    },
                ],
            },
        ],
        ..Default::default()
    };
    let c2 = authority
        .ensure_lineage_for_commit_with_postures(
            execution,
            &mut staged,
            &receipt_at(2, 2),
            &[],
            &c2_changed,
            &c2_input,
        )
        .expect("runtime Merge + relation Split succeeds");
    let c2_ev = c2.artifact().event_batch().events();
    let has = |k, s| c2_ev.iter().any(|e| e.kind() == k && e.subject() == s);
    assert!(has(LineageEventKind::Merge, LineageSubject::Entity));
    assert!(has(LineageEventKind::Split, LineageSubject::Relation));
}
