//! Global private designation cannot turn an aggregate's inputs into consequences.
use super::fixtures::*;
use worth_foundational::facade::{AggregateDefinitionIdentity, AspectIdentity};

fn other(source: &str, member: u32, projected: bool) -> AggregateIdentityContractDeclaration {
    AggregateIdentityContractDeclaration {
        definition: AggregateDefinitionIdentity::from_bytes([43; 32]),
        source: AggregateIdentitySource::DerivedComponents {
            derived_components: identity(source),
        },
        member_kind: KindId(member),
        membership_kind: KindId(42),
        storage: AggregateIdentityStorage {
            aggregate_kind: KindId(40),
            lineage_kind: KindId(41),
            aggregate_aspect: AspectIdentity(3201),
            lineage_aspect: AspectIdentity(3202),
            projected: projected.then_some(AggregateProjectedMemberStorage {
                coordinate_aspect: AspectIdentity(3203),
                membership_aspect: AspectIdentity(3204),
            }),
        },
    }
}
fn storage(
    registry: RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
) -> RelationalSchemaRegistry {
    crate::schema::aggregate_identity::register_aggregate_identity_storage(
        registry,
        &domain().schema,
        SchemaVersionId(1),
        contract.storage,
    )
    .unwrap()
}
fn membership(
    registry: RelationalSchemaRegistry,
    contract: AggregateIdentityContractDeclaration,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    crate::schema::aggregate_identity::register_aggregate_membership(
        registry,
        &domain().schema,
        SchemaVersionId(1),
        contract,
    )
}
fn refused(error: SchemaRegistryError, membership: u32) {
    assert!(
        matches!(error.class, SchemaRegistryErrorClass::InvalidRelationIntegrityDeclaration { kind_id, .. } if kind_id == KindId(membership)),
        "{error:?}"
    );
}

#[test]
fn another_storage_cannot_use_private_membership_but_can_share_real_business_sources() {
    use DerivedRelationNode::*;
    let declared = DerivedRelationDefinitionParts {
        identity: identity("PrivateInput"),
        from: KindId(1).into(),
        to: KindId(1).into(),
        nodes: vec![
            Stored {
                relation: descriptor().membership_kind,
                from: KindId(1),
                to: descriptor().storage.aggregate_kind,
            },
            Reverse { input: 0 },
            Compose { left: 0, right: 1 },
        ],
        laws: None,
    };
    let original = registry()
        .register_derived_relations(vec![declared])
        .unwrap();
    let hostile = other("PrivateInput", 1, false);
    refused(
        membership(storage(original.clone(), &hostile), hostile).unwrap_err(),
        42,
    );
    let lawful = other("Adjacency", 1, true);
    membership(storage(original, &lawful), lawful).unwrap();
}

#[test]
fn later_private_designation_revalidates_previously_admitted_derived_sources() {
    // These are actual installed storage kinds, but no membership has yet made
    // them Native-private. No accepted registry map is removed or rewritten.
    let first = descriptor();
    let registry = storage(source_registry(), &first)
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(30),
            kind_name: "ordinary-before-designation".into(),
            schema_id: domain().schema,
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "ordinary-before-designation:endpoints".into(),
                    allowed_source_kinds: vec![KindId(20)],
                    allowed_target_kinds: vec![KindId(20)],
                    self_edges_allowed: true,
                    cross_context_policy: CrossContextPolicy::Forbid,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap()
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: identity("LateInput"),
            from: KindId(20).into(),
            to: KindId(20).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(30),
                from: KindId(20),
                to: KindId(20),
            }],
            laws: None,
        }])
        .unwrap();
    let second = other("LateInput", 20, false);
    let admitted = membership(storage(registry, &second), second).unwrap();
    assert!(admitted
        .relation_registration(KindId(42))
        .unwrap()
        .relation_integrity
        .aggregate_identity
        .is_some());
    refused(membership(admitted, first).unwrap_err(), 22);
}
