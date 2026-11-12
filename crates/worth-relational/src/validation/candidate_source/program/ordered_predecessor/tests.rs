//! Real registered recipes distinguish each admission premise from its twin.
use super::*;
use crate::schema::data::{
    CardinalityContractDeclaration, MinimumCardinalityEnforcement, PairMinimumSemantics,
};

fn membership(to_one: bool) -> (RelationalSchemaRegistry, DerivedRelationHandle) {
    let (registry, template) = crate::tests::support::managed_cache_schema();
    let mut order = registry
        .relation_registration(crate::identity::data::KindId(3))
        .unwrap()
        .clone();
    order.kind_id = crate::identity::data::KindId(4);
    order.kind_name = "event-predecessor".into();
    order.aspect_contract_declarations = Default::default();
    if to_one {
        order
            .relation_integrity
            .cardinality_contracts
            .push(CardinalityContractDeclaration {
                contract_id: "one-predecessor".into(),
                source_max: Some(1),
                target_max: None,
                pair_max: None,
                source_min: None,
                target_min: None,
                pair_min: None,
                pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
            });
    }
    let mut parts = template.0.path.parts().clone();
    parts.identity.name = "event-membership".into();
    let identity = parts.identity.clone();
    let leaf = parts.nodes[0].clone();
    parts.nodes = vec![
        Node::Stored {
            relation: crate::identity::data::KindId(4),
            from: crate::identity::data::KindId(1),
            to: crate::identity::data::KindId(1),
        },
        leaf.clone(),
        leaf,
        Node::OrderedEvents {
            predecessor: 0,
            introduced: 1,
            retired: 2,
        },
    ];
    let registry = registry
        .register_relation_kind(order)
        .unwrap()
        .register_derived_relations(vec![parts])
        .unwrap();
    let path = registry.derived_relation(&identity).unwrap();
    (registry, path)
}

#[test]
fn ordered_predecessor_admits_only_its_own_to_one_order_relation() {
    let (registry, path) = membership(true);
    let program = registry
        .bind_ordered_predecessor_managed_index_source(&path, vec![], vec![])
        .unwrap();
    assert_eq!(
        program.0.ordered.map(|recipe| recipe.predecessor),
        Some(crate::identity::data::KindId(4))
    );
    assert_eq!(
        registry
            .bind_ordered_predecessor_managed_index_source(&path, vec![], vec![])
            .unwrap(),
        program
    );
    let (registry, path) = membership(false);
    assert_eq!(
        registry.bind_ordered_predecessor_managed_index_source(&path, vec![], vec![]),
        Err(Denial::OrderedPredecessorMustBeToOne)
    );
}

#[test]
fn ordered_predecessor_refuses_non_event_membership_and_foreign_registered_owners() {
    let (registry, ordinary) = crate::tests::support::managed_cache_schema();
    assert_eq!(
        registry.bind_ordered_predecessor_managed_index_source(&ordinary.0.path, vec![], vec![]),
        Err(Denial::OrderedPredecessorMembershipRequired)
    );
    let (registry, path) = membership(true);
    let (other, foreign) = membership(true);
    assert!(other
        .bind_ordered_predecessor_managed_index_source(&foreign, vec![], vec![])
        .is_ok());
    assert_eq!(
        registry.bind_ordered_predecessor_managed_index_source(&foreign, vec![], vec![]),
        Err(Denial::ForeignDefinition)
    );
    assert!(registry
        .bind_ordered_predecessor_managed_index_source(&path, vec![], vec![])
        .is_ok());
}
