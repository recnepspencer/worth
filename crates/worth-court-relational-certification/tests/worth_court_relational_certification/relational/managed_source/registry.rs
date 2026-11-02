//! One actual Source schema with optional declared Struct-field projection profiles.
use super::*;
use worth_foundational::facade::{
    aspects, AbsenceLaw, AspectBinding, AspectContract, AspectEquivalenceBasis,
    AspectEvolutionPolicy, AspectFieldLocator, AspectIdentity, AspectKey, AspectMaskContract,
    CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
};

pub(super) fn key() -> AspectKey {
    AspectKey::new("history-data").unwrap()
}
pub(super) fn field(name: &str) -> FieldKey {
    FieldKey::new(name).unwrap()
}
pub(super) fn locator(name: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        key(),
        CanonicalFieldPath::single(field(name)),
    )
}
pub(super) fn contract() -> AspectContract {
    let shape = aspects()
        .struct_fields()
        .optional("value", ScalarAspectType::String)
        .optional("other", ScalarAspectType::String)
        .finish()
        .unwrap();
    aspects()
        .contract()
        .for_key(key())
        .identified_by(AspectIdentity(0x9190_0401))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_with(
            shape,
            AspectMaskContract::struct_fields(),
            AbsenceLaw::Optional,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
}
fn declared_aspect() -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: field("history-data"),
        },
        contract: contract(),
    }
}

pub(crate) fn registry() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    profile(Profile::Membership)
}
#[derive(Clone, Copy)]
pub(super) enum Profile {
    Membership,
    Fields,
    SchemaFieldsOnly,
}
pub(super) fn profile(profile: Profile) -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let mut registry = RelationalSchemaRegistry::new();
    let schema = SchemaId("managed-source-court".into());
    for (kind, name) in [(1, "directory"), (4, "item")] {
        registry = registry
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(kind),
                kind_name: name.into(),
                schema_id: schema.clone(),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: if matches!(profile, Profile::Membership) {
                    Default::default()
                } else {
                    KindAspectContractDeclarations::new(vec![declared_aspect()])
                },
            })
            .unwrap();
    }
    registry = registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "contains".into(),
            schema_id: schema.clone(),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "contains-endpoints".into(),
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
    let identity = DerivedRelationIdentity {
        schema,
        version: SchemaVersionId(1),
        name: "directory-items".into(),
    };
    registry = registry
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: identity.clone(),
            from: KindId(1).into(),
            to: KindId(4).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(2),
                from: KindId(1),
                to: KindId(4),
            }],
            laws: None,
        }])
        .unwrap();
    let mut fields = vec![];
    let mut routes = vec![];
    if matches!(profile, Profile::Fields) {
        let reverse = DerivedRelationIdentity {
            name: "item-owner".into(),
            ..identity.clone()
        };
        registry = registry
            .register_derived_relations(vec![DerivedRelationDefinitionParts {
                identity: reverse.clone(),
                from: KindId(4).into(),
                to: KindId(1).into(),
                nodes: vec![
                    DerivedRelationNode::Stored {
                        relation: KindId(2),
                        from: KindId(1),
                        to: KindId(4),
                    },
                    DerivedRelationNode::Reverse { input: 0 },
                ],
                laws: None,
            }])
            .unwrap();
        fields.push(ManagedIndexSourceField {
            locator: locator("value"),
        });
        routes.push(
            ManagedIndexSourceProjectionRoute::new(
                "owner".into(),
                registry.derived_relation(&reverse).unwrap(),
                vec![ManagedIndexSourceField {
                    locator: locator("value"),
                }],
            )
            .unwrap(),
        );
    }
    let handle = registry.derived_relation(&identity).unwrap();
    let program = registry
        .bind_managed_index_source(&handle, fields, routes)
        .unwrap();
    (registry, program)
}
