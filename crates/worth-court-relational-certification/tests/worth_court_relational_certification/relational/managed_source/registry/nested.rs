//! Independent observes adjacency makes empty nested ranges real.
use super::*;
pub(super) fn extend(
    mut registry: RelationalSchemaRegistry,
    identity: &DerivedRelationIdentity,
    routes: &mut Vec<ManagedIndexSourceProjectionRoute>,
) -> RelationalSchemaRegistry {
    registry = registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(3),
            kind_name: "observes".into(),
            schema_id: identity.schema.clone(),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "observes-endpoints".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(4)],
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::Forbid,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap();
    let observed = DerivedRelationIdentity {
        name: "directory-observes".into(),
        ..identity.clone()
    };
    registry = registry
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: observed.clone(),
            from: KindId(1).into(),
            to: KindId(4).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(3),
                from: KindId(1),
                to: KindId(4),
            }],
            laws: None,
        }])
        .unwrap();
    routes.push(
        ManagedIndexSourceProjectionRoute::nested(
            "observed".into(),
            0,
            registry.derived_relation(&observed).unwrap(),
            vec![ManagedIndexSourceField {
                locator: locator("value"),
            }],
        )
        .unwrap(),
    );
    let reverse = DerivedRelationIdentity {
        name: "item-owner".into(),
        ..identity.clone()
    };
    routes.push(
        ManagedIndexSourceProjectionRoute::nested(
            "observed-owners".into(),
            1,
            registry.derived_relation(&reverse).unwrap(),
            vec![ManagedIndexSourceField {
                locator: locator("value"),
            }],
        )
        .unwrap(),
    );
    registry
}
