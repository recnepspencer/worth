//! Native transition DATA from the actual A/B schema observations.
use super::schema::{LaterIdentity, LaterRecord, LaterRecords};
use worth_query_decl::facade::application_schema::fragment::FieldSelection;
pub(in crate::host::application_home::program_transition) fn schema_change(
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
) -> ProposedSchemaTransition {
    use worth_query_host::facade::home_program_transition::*;
    let (before, before_version) = source
        .schema_registry()
        .authoritative_schema_basis()
        .unwrap()
        .unwrap();
    let (after, after_version) = target
        .schema_registry()
        .authoritative_schema_basis()
        .unwrap()
        .unwrap();
    ProposedSchemaTransition {
        source_schema_id: before,
        source_schema_version_id: before_version,
        target_schema_id: after.clone(),
        target_schema_version_id: after_version,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::EntityKind,
                after,
                after_version,
                target.entity_kind("LaterRecord"),
                "LaterRecord",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail:
                    "add an authored kind and label beside the unchanged original Account contracts"
                        .into(),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    }
}

use worth_query_host::facade::home_program_transition::{
    ProposedSchemaTransition, WorthQueryHomeProgramTransitionReadSource,
    WorthQueryHomeProgramTransitionTarget,
};

pub(in crate::host::application_home::program_transition) fn body(
    target: &WorthQueryHomeProgramTransitionTarget<'_>,
) -> worth_query_host::facade::home_program_transition::WorkerIntentBatch {
    use worth_query_host::facade::home_program_transition::*;
    WorkerIntentBatch::new("create-in-new-b-kind").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: target.entity_kind("LaterRecord").unwrap(),
            client_key: ClientKey::raw("home-successor-row"),
            fields: AspectFieldPatch::from_locator(
                target
                    .planned_field(
                        "LaterRecord",
                        FieldSelection::<LaterRecords, LaterRecord, LaterIdentity, String>::at(
                            "Label",
                        )
                        .aspect(),
                        "Label",
                    )
                    .unwrap(),
                AspectValue::String("created-by-target".into()),
            ),
        }),
    ))
}
