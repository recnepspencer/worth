//! Actual successor schema and authored row, distinct from the incumbent.
use worth_foundational::facade::{aspects, AspectIdentity, AspectKey, ScalarAspectType};
use worth_relational::facade::{identity::KindId, schema::*, transactions::*};

pub(super) fn registry() -> RelationalSchemaRegistry {
    crate::schema_transition::registry(crate::schema_transition::SOURCE)
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(2000),
            kind_name: "indexed-target".into(),
            schema_id: SchemaId("oracle".into()),
            schema_version_id: crate::schema_transition::SOURCE,
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
                        field: crate::batches::field_key("label"),
                    },
                    contract: aspects()
                        .contract()
                        .for_key(AspectKey::new("target-label").unwrap())
                        .identified_by(AspectIdentity(2000))
                        .at_revision(aspects().vocabulary().revision(1))
                        .struct_aspect(
                            aspects()
                                .struct_fields()
                                .required("label", ScalarAspectType::String)
                                .finish()
                                .unwrap(),
                        ),
                },
            ]),
        })
        .unwrap()
}
pub(super) fn proposed() -> ProposedSchemaTransition {
    let schema = SchemaId("oracle".into());
    let version = crate::schema_transition::SOURCE;
    ProposedSchemaTransition {
        source_schema_id: schema.clone(),
        source_schema_version_id: version,
        target_schema_id: schema.clone(),
        target_schema_version_id: version,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::EntityKind,
                schema,
                version,
                Some(KindId(2000)),
                "indexed-target",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail: "add an indexed authored kind beside the exact source".into(),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    }
}
pub(super) fn body() -> WorkerIntentBatch {
    WorkerIntentBatch::new("create-indexed-target").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: KindId(2000),
            client_key: worth_relational::facade::symbols::ClientKey::raw("indexed-target"),
            fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Planned,
                    AspectKey::new("target-label").unwrap(),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        crate::batches::field_key("label"),
                    ),
                ),
                worth_foundational::facade::AspectValue::String("target-value".into()),
            )])),
        }),
    ))
}
pub(super) fn definition(
    name: &str,
    aspect: &str,
    field: &str,
) -> worth_relational::facade::indexes::DerivedIndexDefinition {
    worth_relational::facade::indexes::DerivedIndexDefinition {
        index_id: worth_relational::facade::indexes::DerivedIndexId(999),
        name: name.into(),
        kind: worth_relational::facade::indexes::DerivedIndexKind::EntityField {
            field_locator: worth_foundational::facade::AspectFieldLocator::new(
                worth_foundational::facade::LocatorAuthority::Authoritative,
                AspectKey::new(aspect).unwrap(),
                worth_foundational::facade::CanonicalFieldPath::single(crate::batches::field_key(
                    field,
                )),
            ),
        },
        branch_scoped: true,
    }
}
