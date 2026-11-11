//! Generic event and row kinds reuse the existing Native Source court schema.
use super::*;
pub(super) fn parts() -> DerivedRelationDefinitionParts {
    DerivedRelationDefinitionParts {
        identity: DerivedRelationIdentity {
            schema: SchemaId("managed-source-court".into()),
            version: SchemaVersionId(1),
            name: "ordered-events".into(),
        },
        from: KindId(1).into(),
        to: KindId(4).into(),
        laws: None,
        nodes: vec![
            DerivedRelationNode::Stored {
                relation: KindId(5),
                from: KindId(1),
                to: KindId(1),
            },
            DerivedRelationNode::Stored {
                relation: KindId(2),
                from: KindId(1),
                to: KindId(4),
            },
            DerivedRelationNode::Stored {
                relation: KindId(3),
                from: KindId(1),
                to: KindId(4),
            },
            DerivedRelationNode::OrderedEvents {
                predecessor: 0,
                introduced: 1,
                retired: 2,
            },
        ],
    }
}
pub(super) fn base() -> RelationalSchemaRegistry {
    base_with_predecessor_laws(vec![])
}
pub(super) fn base_with_predecessor_laws(
    laws: Vec<AcyclicityContractDeclaration>,
) -> RelationalSchemaRegistry {
    let (mut registry, _) = super::super::registry::registry();
    for (kind, to) in [(3, 4), (5, 1)] {
        registry =
            registry
                .register_relation_kind(RelationKindRegistration {
                    kind_id: KindId(kind),
                    kind_name: format!("event-{kind}"),
                    schema_id: SchemaId("managed-source-court".into()),
                    schema_version_id: SchemaVersionId(1),
                    cross_context_policy: CrossContextPolicy::Forbid,
                    cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                    aspect_contract_declarations: Default::default(),
                    relation_integrity: RelationIntegrityDeclarations::new(
                        vec![EndpointKindContractDeclaration {
                            contract_id: format!("event-{kind}").into(),
                            allowed_source_kinds: vec![KindId(1)],
                            allowed_target_kinds: vec![KindId(to)],
                            self_edges_allowed: true,
                            cross_context_policy: CrossContextPolicy::Forbid,
                        }],
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                    )
                    .with_acyclicity_contracts(if kind == 5 { laws.clone() } else { vec![] }),
                })
                .unwrap();
    }
    registry
}
pub(in super::super::super) fn profile() -> (
    RelationalSchemaRegistry,
    ManagedIndexSourceProgram,
    DerivedRelationHandle,
) {
    let parts = parts();
    let registry = base()
        .register_derived_relations(vec![parts.clone()])
        .unwrap();
    let path = registry.derived_relation(&parts.identity).unwrap();
    let program = registry
        .bind_managed_index_source(&path, vec![], vec![])
        .unwrap();
    (registry, program, path)
}
