//! Refusals preserve foreign relation ownership and external endpoint generations.
use super::graph_boundary::rewire;
use super::*;
use worth_relational::facade::branch::RelationalOwnedModelSelectionDenial as GraphDenial;

#[test]
fn an_old_outgoing_relation_now_owned_by_an_outside_source_cannot_be_rewritten() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let model = create_entity(execution, &runtime, "model");
    let outside = create_entity(execution, &runtime, "outside");
    let catalog = create_entity(execution, &runtime, "catalog");
    let outgoing =
        create_relation_of_kind(execution, &runtime, model, catalog, "reference", KindId(4));
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    rewire(execution, &runtime, outgoing, outside, catalog);
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(
        matches!(runtime.prepare_owned_model_graph_selection(execution, &current, &selected, &ownership(), &ownership(), model), Err(GraphDenial::ForeignRelationSource { relation, owner }) if relation == outgoing && owner == outside)
    );
    assert_eq!(
        begin(&runtime)
            .read_relation(execution, outgoing)
            .unwrap()
            .base()
            .unwrap()
            .source,
        outside
    );
}

#[test]
fn a_selected_outgoing_reference_cannot_restore_an_unavailable_external_endpoint() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let model = create_entity(execution, &runtime, "model");
    let catalog = create_entity(execution, &runtime, "catalog");
    let outgoing =
        create_relation_of_kind(execution, &runtime, model, catalog, "reference", KindId(4));
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = begin(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("remove-reference-and-catalog")
                .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                    DeleteRelationIntent {
                        relation_id: outgoing,
                    },
                )))
                .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                    DeleteEntityIntent { entity_id: catalog },
                ))),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &result);
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(
        matches!(runtime.prepare_owned_model_graph_selection(execution, &current, &selected, &ownership(), &ownership(), model), Err(GraphDenial::SelectedEndpointUnavailable { relation, endpoint }) if relation == outgoing && endpoint == catalog)
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        current.descriptor().clone()
    );
}

#[test]
fn a_foreign_incoming_relation_cannot_be_severed_when_its_kind_has_no_deletion_contract() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let model = create_entity(execution, &runtime, "model");
    let owner = create_entity(execution, &runtime, "foreign-owner");
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let member = member(execution, &runtime, "new-member");
    create_relation(execution, &runtime, model, member, "owns");
    let incoming = create_relation_of_kind(
        execution,
        &runtime,
        owner,
        member,
        "uncontracted-reference",
        KindId(6),
    );
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let GraphDenial::ForeignIncomingRelation(foreign) = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &current,
            &selected,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap_err()
    else {
        panic!("outside source custody still forbids severing this relation")
    };
    assert_eq!(
        (foreign.entity(), foreign.relation(), foreign.owner()),
        (member, incoming, owner)
    );
    assert!(foreign.deletion_contracts().is_empty());
    assert!(begin(&runtime)
        .read_relation(execution, incoming)
        .unwrap()
        .effective()
        .is_some());
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        current.descriptor().clone()
    );
}
