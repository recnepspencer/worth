//! The navigation entity boundary is derived independently at both real bases.
use super::*;
use std::collections::BTreeSet;

#[test]
fn entity_boundary_rejects_foreign_selected_basis_before_reading_current_material() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
    let before_reads = graph.source.reads.load(Ordering::Relaxed);
    let foreign = runtime();
    let (_, selected) = foreign
        .observe_branch(&foreign.main_branch_identity())
        .unwrap();
    let (_, current) = graph
        .runtime
        .observe_branch(&graph.runtime.main_branch_identity())
        .unwrap();
    assert!(matches!(
        graph.runtime.prepare_owned_model_entity_boundary(
            execution,
            &current,
            &selected,
            &ownership(),
            graph.model
        ),
        Err(Denial::ForeignBasis)
    ));
    assert_eq!(graph.source.reads.load(Ordering::Relaxed), before_reads);
}

#[test]
fn model_entity_boundary_unites_old_and_current_ownership_without_outside_payloads() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let old = member(execution, &runtime, "old");
    let child = member(execution, &runtime, "child");
    let new = member(execution, &runtime, "new");
    let outside = member(execution, &runtime, "outside");
    let edge = create_relation(execution, &runtime, a, old, "old-owner");
    create_relation(execution, &runtime, old, child, "old-child");
    let other_edge = create_relation(execution, &runtime, b, new, "new-owner");
    create_relation(execution, &runtime, b, outside, "outside-owner");
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    // A stored relation move replaces A's old subtree with a different member.
    let mut transaction = begin(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("move-members")
                .push(MutationIntent::Relation(
                    RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                        relation_id: edge,
                        kind_id: KindId(2),
                        source: EntityReference::Existing(a),
                        target: EntityReference::Existing(new),
                    }),
                ))
                .push(MutationIntent::Relation(
                    RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                        relation_id: other_edge,
                        kind_id: KindId(2),
                        source: EntityReference::Existing(b),
                        target: EntityReference::Existing(old),
                    }),
                )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let boundary = runtime
        .prepare_owned_model_entity_boundary(execution, &current, &selected, &ownership(), a)
        .unwrap();
    assert_eq!(boundary.model(), a);
    assert_eq!(boundary.current_basis().descriptor(), current.descriptor());
    assert_eq!(
        boundary.selected_basis().descriptor(),
        selected.descriptor()
    );
    assert_eq!(boundary.current_entities(), &BTreeSet::from([a, new]));
    assert_eq!(
        boundary.selected_entities(),
        &BTreeSet::from([a, old, child])
    );
    assert_eq!(
        boundary.entities().collect::<BTreeSet<_>>(),
        BTreeSet::from([a, old, child, new])
    );
    assert!(!boundary.entities().any(|id| id == b || id == outside));
    // A later ordinary edit does not rewrite either retained answer.
    create_entity(execution, &runtime, "later-unrelated-model");
    assert_eq!(
        boundary.selected_entities(),
        &BTreeSet::from([a, old, child])
    );
    assert!(matches!(
        runtime.prepare_owned_model_entity_boundary(
            execution,
            &current,
            &selected,
            &ownership(),
            a
        ),
        Err(Denial::TargetNotCurrent)
    ));
}
#[test]
fn entity_boundary_refuses_wrong_model_kind_without_changing_selected_basis() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let graph = graph(execution);
    let (_, basis) = graph
        .runtime
        .observe_branch(&graph.runtime.main_branch_identity())
        .unwrap();
    let boundary = graph
        .runtime
        .prepare_owned_model_entity_boundary(execution, &basis, &basis, &ownership(), graph.model)
        .unwrap();
    drop(boundary);
    assert!(
        matches!(graph.runtime.prepare_owned_model_entity_boundary(execution, &basis, &basis, &ownership(), graph.nested), Err(Denial::ModelRootKind { model, actual: KindId(3) }) if model == graph.nested)
    );
    assert_eq!(
        graph
            .runtime
            .observe_branch(&graph.runtime.main_branch_identity())
            .unwrap()
            .0,
        basis.descriptor().clone()
    );
}
