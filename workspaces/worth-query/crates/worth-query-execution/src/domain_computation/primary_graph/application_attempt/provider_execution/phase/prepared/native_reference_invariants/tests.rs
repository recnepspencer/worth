use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationCreationPartition,
    tests::fixture::installed_authorization_world,
    workflow::operation_output::append_operation_outputs,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture;
use worth_relational::facade::{
    identity::PartitionId, symbols::ClientKey, transactions::CreatedEntityRef,
};

#[test]
fn endpoint_reference_selects_only_matching_entity_applicability() {
    let referenced = BTreeSet::from(["Account"]);
    assert!(applies_to_references(
        &[ApplicationInvariantScopeTarget::Entity("Account".into())],
        &referenced,
    ));
    for unrelated in [
        ApplicationInvariantScopeTarget::Entity("Other".into()),
        ApplicationInvariantScopeTarget::Relation("Account".into()),
    ] {
        assert!(
            !applies_to_references(&[unrelated], &referenced),
            "an unrelated entity or same-spelling relation cannot enter the invocation set"
        );
    }
    assert!(!applies_to_references(
        &[ApplicationInvariantScopeTarget::Entity("Account".into())],
        &BTreeSet::new(),
    ));
}

#[test]
fn native_output_reference_selects_endpoint_while_ordinary_relations_do_not() {
    let world = installed_authorization_world(false);
    let layout = &world.application.primary_provider.graph.layout;
    let transition = CreatedEntityRef {
        partition_id: PartitionId(1),
        kind_id: layout.workflow().transition.entity_kind,
        client_key: ClientKey::raw("settlement"),
    };
    let mut effects = vec![Effect::CreateEntity {
        kind: transition.kind_id,
        key: "settlement".into(),
        fields: BTreeMap::new(),
        partition: WorthQueryApplicationCreationPartition::Context(transition.partition_id),
    }];
    let target = EntityReference::Created(CreatedEntityRef {
        partition_id: transition.partition_id,
        kind_id: layout.entity_kind("Account").unwrap(),
        client_key: ClientKey::raw("account"),
    });
    append_operation_outputs(
        layout.workflow(),
        transition,
        "source-binding",
        [(
            "account",
            ApplicationMutationOutputPosture::Create,
            "Account",
            target,
        )]
        .into_iter(),
        &mut effects,
    );
    assert_eq!(
        referenced_entities(layout, &effects).unwrap(),
        BTreeSet::from(["Account"])
    );
    let Effect::CreateRelation { kind, .. } = effects.last_mut().unwrap() else {
        panic!()
    };
    *kind = layout.relation("AccountOwner").unwrap().kind;
    assert!(referenced_entities(layout, &effects).unwrap().is_empty());
}

#[test]
fn native_reference_without_matching_output_entry_is_refused() {
    let world = installed_authorization_world(false);
    let layout = &world.application.primary_provider.graph.layout;
    let entry = EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId(1),
        kind_id: layout.workflow().operation_output.entity_kind,
        client_key: ClientKey::raw("missing-entry"),
    });
    let effects = [Effect::CreateRelation {
        kind: layout.workflow().operation_output.target,
        key: "native-target".into(),
        from: entry.clone(),
        to: entry,
    }];
    assert_eq!(
        referenced_entities(layout, &effects).unwrap_err(),
        "native output reference has no output entry"
    );
}
