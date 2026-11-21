use super::super::*;

pub(in crate::tests::complexity::contracts::commit_budgets) fn schema_transition_for_subscriber_impact(
    target_schema_version_id: SchemaVersionId,
    subscriber_impact: crate::schema::data::SchemaSubscriberImpact,
) -> crate::schema::data::ProposedSchemaTransition {
    crate::schema::data::ProposedSchemaTransition {
        source_schema_id: crate::schema::data::SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(target_schema_version_id.0 - 1),
        target_schema_id: crate::schema::data::SchemaId("test".to_string()),
        target_schema_version_id,
        diff_atoms: vec![crate::schema::data::SchemaDiffAtom::new(
            crate::schema::data::SchemaElementRef::new(
                crate::schema::data::SchemaElementKind::Field,
                crate::schema::data::SchemaId("test".to_string()),
                target_schema_version_id,
                Some(KindId(1)),
                "tag",
            ),
            vec![
                crate::schema::data::SchemaStratum::StructuralShape,
                crate::schema::data::SchemaStratum::PublicationContract,
            ],
            crate::schema::data::SchemaPublicationImpact::ObservableSurfaceChanged,
            subscriber_impact,
            crate::schema::data::HistoricalInterpretationSensitivity::NotSensitive,
            crate::schema::data::SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(match subscriber_impact {
            crate::schema::data::SchemaSubscriberImpact::ConsumableSurfaceChanged => {
                crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable
            }
            crate::schema::data::SchemaSubscriberImpact::ContractUpgradeRequired => {
                crate::schema::data::SubscriberBoundaryVisibility::VisibleRequiresContractUptake
            }
            _ => crate::schema::data::SubscriberBoundaryVisibility::NotVisible,
        })],
    }
}
