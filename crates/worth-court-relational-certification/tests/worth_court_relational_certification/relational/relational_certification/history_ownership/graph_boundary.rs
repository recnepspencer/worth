//! D1117 restoration material uses actual graph facts and outside source custody.
use super::*;
use worth_relational::facade::{
    branch::RelationalOwnedModelSelectionDenial as GraphDenial, identity::RelationId,
};

pub(super) fn rewire(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    id: RelationId,
    source: EntityId,
    target: EntityId,
) {
    let mut transaction = begin(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("rewire-reference").push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: id,
                    kind_id: KindId(4),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}

#[test]
fn prepared_graph_selects_old_outgoing_reference_and_excludes_foreign_incoming_relation() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let model = create_entity(execution, &runtime, "model");
    let outside = create_entity(execution, &runtime, "outside");
    let catalog = create_entity(execution, &runtime, "catalog");
    let newer_catalog = create_entity(execution, &runtime, "newer-catalog");
    let member = member(execution, &runtime, "member");
    create_relation(execution, &runtime, model, member, "owns");
    let outgoing =
        create_relation_of_kind(execution, &runtime, member, catalog, "reference", KindId(4));
    let incoming = create_relation_of_kind(
        execution,
        &runtime,
        outside,
        member,
        "foreign-reference",
        KindId(4),
    );
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    rewire(execution, &runtime, outgoing, member, newer_catalog);
    rewire(execution, &runtime, incoming, outside, model);
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let selection = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &current,
            &selected,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap();
    assert_eq!(selection.selected_relations()[&outgoing].target, catalog);
    assert_eq!(
        selection.current_relations()[&outgoing].target,
        newer_catalog
    );
    assert!(!selection.current_relations().contains_key(&incoming));
    assert!(!selection.selected_relations().contains_key(&incoming));
    assert!(!selection.relations_to_retire().any(|id| id == incoming));
    assert!(!selection
        .entities()
        .entities()
        .any(|id| [outside, catalog, newer_catalog].contains(&id)));
    let read = begin(&runtime).read_relation(execution, incoming).unwrap();
    assert_eq!(read.base().unwrap().source, outside);
    assert_eq!(read.base().unwrap().target, model);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        current.descriptor().clone()
    );
}

#[test]
fn retiring_a_current_only_member_refuses_the_foreign_relation_and_its_real_contract() {
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
        "foreign-reference",
        KindId(4),
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
        panic!("foreign incoming edge must prevent retirement")
    };
    assert_eq!(
        (
            foreign.entity(),
            foreign.relation(),
            foreign.owner(),
            foreign.kind()
        ),
        (member, incoming, owner, KindId(4))
    );
    assert_eq!(foreign.deletion_contracts().len(), 1);
    assert_eq!(
        foreign.deletion_contracts()[0].contract_id.as_str(),
        "References:deletion"
    );
    assert_eq!(
        foreign.deletion_contracts()[0].mode,
        EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations
    );
    assert!(begin(&runtime)
        .read_entity(execution, member)
        .unwrap()
        .effective()
        .is_some());
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

#[test]
fn an_ownership_move_includes_the_other_declared_relation_kind_on_a_selected_only_member() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let model = create_entity(execution, &runtime, "model");
    let other = create_entity(execution, &runtime, "other-model");
    let member = member(execution, &runtime, "member");
    let old = create_relation(execution, &runtime, model, member, "original-owner");
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = begin(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("move-owner").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: old }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &result);
    let moved = create_relation_of_kind(execution, &runtime, other, member, "new-owner", KindId(5));
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let document = ownership();
    let other_domain = RelationalModelOwnership::new(vec![RelationalModelOwnershipDomain {
        root: KindId(1),
        entities: vec![KindId(3)],
        relations: vec![KindId(2), KindId(5)],
    }]);
    let installed = RelationalModelOwnership::combine([&document, &other_domain]);
    let selection = runtime
        .prepare_owned_model_graph_selection(
            execution, &current, &selected, &document, &installed, model,
        )
        .unwrap();
    assert!(!selection.entities().current_entities().contains(&member));
    assert!(selection.entities().selected_entities().contains(&member));
    assert_eq!(selection.current_relations()[&moved].source, other);
    assert_eq!(selection.selected_relations()[&old].source, model);
    assert!(selection.relations_to_retire().any(|id| id == moved));
    assert!(!selection.entities().entities().any(|id| id == other));
    assert!(begin(&runtime)
        .read_relation(execution, moved)
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
