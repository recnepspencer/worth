//! Outside owners keep liveness, exclusivity and their own commit invariants.
use super::transfer_fixture::*;
use super::*;
use worth_relational::facade::{
    branch::{RelationalModelOwnershipConflict, RelationalOwnedModelSelectionDenial},
    mvcc::RelationalOwnedGraphSelectionPreparationError,
};
#[test]
fn a_retired_selected_owner_refuses_with_both_member_and_owner_and_no_effect() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let e = member(execution, &runtime, "member");
    let edge = create_relation(execution, &runtime, a, e, "owner");
    let at_a = basis(&runtime);
    move_to(execution, &runtime, edge, b, e);
    let at_b = basis(&runtime);
    let committed = select(execution, &runtime, a, &at_a).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let mut tx = begin(&runtime);
    tx.push_batch(
        WorkerIntentBatch::new("retire-empty-outside-owner").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: b }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let before = basis(&runtime);
    let error = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &before,
            &at_b,
            &ownership(),
            &ownership(),
            a,
        )
        .unwrap_err();
    assert!(
        matches!(error, RelationalOwnedModelSelectionDenial::SelectedOwnerUnavailable {
        entity, owner, source, relation,
    } if entity == e && owner == b && source == b && relation == edge),
        "{error:?}"
    );
    assert_eq!(basis(&runtime).descriptor(), before.descriptor());
    assert_owner(execution, &runtime, e, edge, a, &[a]);
    assert!(begin(&runtime)
        .read_entity(execution, b)
        .unwrap()
        .base()
        .is_none());
}
#[test]
fn a_second_owner_candidate_after_selection_refuses_the_true_member_and_keeps_b_as_owner() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let e = member(execution, &runtime, "member");
    let edge = create_relation(execution, &runtime, a, e, "owner");
    let at_a = basis(&runtime);
    move_to(execution, &runtime, edge, b, e);
    let at_b = basis(&runtime);
    for selected in [&at_a, &at_b] {
        let committed = select(execution, &runtime, a, selected).unwrap();
        release_test_commit_snapshot(&runtime, &committed);
    }
    let before = basis(&runtime);
    let mut tx = begin(&runtime);
    tx.push_batch(
        WorkerIntentBatch::new("second-owner-candidate").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("second-owner"),
                source: EntityReference::Existing(a),
                target: EntityReference::Existing(e),
                fields: AspectFieldPatch::default(),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let error = tx.commit(execution, &runtime).unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("{error:?}")
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::CustomInvariantViolation {
                refusal:
                    worth_relational::facade::runtime::CustomInvariantRefusal::DeclaredValue {
                        value_identity,
                        bytes,
                    },
                ..
            },
        ..
    } = error.class
    else {
        panic!("{error:?}")
    };
    assert_eq!(
        value_identity,
        RelationalModelOwnershipConflict::VALUE_IDENTITY
    );
    assert_eq!(
        RelationalModelOwnershipConflict::decode(&bytes)
            .unwrap()
            .subject,
        e
    );
    assert_eq!(basis(&runtime).descriptor(), before.descriptor());
    assert_owner(execution, &runtime, e, edge, b, &[a, b]);
}
#[test]
fn the_selected_outside_owners_current_capacity_refuses_graph_restoration_atomically() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = capacity_limited_runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let e = member(execution, &runtime, "member");
    let edge = create_relation(execution, &runtime, a, e, "owner");
    move_to(execution, &runtime, edge, b, e);
    let at_b = basis(&runtime);
    move_to(execution, &runtime, edge, a, e);
    let other = member(execution, &runtime, "new-b-member");
    let other_edge = create_relation(execution, &runtime, b, other, "b-now-full");
    let before = basis(&runtime);
    let error = select(execution, &runtime, a, &at_b).unwrap_err();
    let RelationalOwnedGraphSelectionPreparationError::Preparation(
        TransactionCommitError::Conflict { error, .. },
    ) = error
    else {
        panic!("{error:?}")
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id,
                relation_kind_id,
                entity_id,
                count,
                limit,
                ..
            },
        ..
    } = error.class
    else {
        panic!("{error:?}")
    };
    assert_eq!(contract_id, ContractId::new("ModelMembers:capacity"));
    assert_eq!(relation_kind_id, KindId(2));
    assert_eq!(entity_id, EntityReference::Existing(b));
    assert_eq!((count, limit), (2, 1));
    assert_eq!(basis(&runtime).descriptor(), before.descriptor());
    assert_owner(execution, &runtime, e, edge, a, &[a, b]);
    assert_owner(execution, &runtime, other, other_edge, b, &[a, b]);
}
fn capacity_limited_runtime() -> RelationalRuntime {
    let source = registry();
    let mut registry = RelationalSchemaRegistry::new();
    for kind in [KindId(1), KindId(3)] {
        registry = registry
            .register_entity_kind(source.entity_registration(kind).unwrap().clone())
            .unwrap();
    }
    for kind in [KindId(2), KindId(4), KindId(5), KindId(6)] {
        let mut registration = source.relation_registration(kind).unwrap().clone();
        if kind == KindId(2) {
            registration.relation_integrity = RelationIntegrityDeclarations::new(
                vec![],
                vec![CardinalityContractDeclaration {
                    contract_id: ContractId::new("ModelMembers:capacity"),
                    source_max: Some(1),
                    target_max: None,
                    pair_max: None,
                    source_min: None,
                    target_min: None,
                    pair_min: None,
                    pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
                    minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
                }],
                vec![],
                vec![],
                vec![],
            );
        }
        registry = registry.register_relation_kind(registration).unwrap();
    }
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            RelationalSchemaRegistry::new(),
            vec![ownership().invariant_registration().unwrap()],
        )
        .unwrap();
    runtime
}

#[test]
fn a_reparented_selected_source_refuses_instead_of_giving_the_member_to_the_wrong_document() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let c = create_entity(execution, &runtime, "c");
    let parent = member(execution, &runtime, "selected-parent");
    let e = member(execution, &runtime, "member");
    let parent_edge = create_relation(execution, &runtime, b, parent, "parent-owner");
    let edge = create_relation(execution, &runtime, a, e, "owner");
    move_to(execution, &runtime, edge, parent, e);
    let at_b = basis(&runtime);
    move_to(execution, &runtime, edge, a, e);
    move_to(execution, &runtime, parent_edge, c, parent);
    let before = basis(&runtime);
    let error = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &before,
            &at_b,
            &ownership(),
            &ownership(),
            a,
        )
        .unwrap_err();
    assert!(
        matches!(error, RelationalOwnedModelSelectionDenial::SelectedOwnerChanged {
        entity, owner, source, actual_owner, relation,
    } if entity == e && owner == b && source == parent && actual_owner == Some(c) && relation == edge),
        "{error:?}"
    );
    assert_eq!(basis(&runtime).descriptor(), before.descriptor());
    assert_owner(execution, &runtime, e, edge, a, &[a, b, c]);
    assert_owner(execution, &runtime, parent, parent_edge, c, &[a, b, c]);
}

#[test]
fn a_transferred_sources_unowned_reference_cannot_be_severed_to_retire_another_member() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let source = member(execution, &runtime, "transferred");
    let edge = create_relation(execution, &runtime, b, source, "owner");
    let at_b = basis(&runtime);
    move_to(execution, &runtime, edge, a, source);
    let current_only = member(execution, &runtime, "current-only");
    create_relation(execution, &runtime, a, current_only, "current-only-owner");
    let reference = create_relation_of_kind(
        execution,
        &runtime,
        source,
        current_only,
        "outside-reference",
        KindId(4),
    );
    let before = basis(&runtime);
    let error = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &before,
            &at_b,
            &ownership(),
            &ownership(),
            a,
        )
        .unwrap_err();
    let foreign = error
        .foreign_incoming_relation()
        .expect("the transferred source owns its ordinary relation");
    assert_eq!(foreign.entity(), current_only);
    assert_eq!(foreign.owner(), source);
    assert_eq!(foreign.relation(), reference);
    assert_eq!(basis(&runtime).descriptor(), before.descriptor());
    assert_owner(execution, &runtime, source, edge, a, &[a, b]);
}
