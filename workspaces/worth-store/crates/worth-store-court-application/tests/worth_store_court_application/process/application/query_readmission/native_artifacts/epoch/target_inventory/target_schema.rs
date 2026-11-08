//! Add a real empty kind while preserving every admitted predecessor contract.
use worth_relational::facade::{identity::KindId, runtime::RelationalRuntime, schema::*};

pub(super) fn registry(native: &RelationalRuntime) -> RelationalSchemaRegistry {
    let registry = native.config().schema.registry.clone();
    let (schema, version) = registry.authoritative_schema_basis().unwrap().unwrap();
    registry
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(2000),
            kind_name: "epoch-target-kind".into(),
            schema_id: schema,
            schema_version_id: version,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap()
}
pub(super) fn transition(native: &RelationalRuntime) -> ProposedSchemaTransition {
    let (schema, version) = native
        .config()
        .schema
        .registry
        .authoritative_schema_basis()
        .unwrap()
        .unwrap();
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
                "epoch-target-kind",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail: "add an empty kind alongside the actual predecessor contracts".into(),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    }
}
