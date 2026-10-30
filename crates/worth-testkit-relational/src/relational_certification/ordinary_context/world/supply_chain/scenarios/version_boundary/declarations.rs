pub use crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale;
pub use crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion;
pub use worth_foundational::facade::FieldKey;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName,
    crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline,
};
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind,
};
pub use {
    worth_relational::facade::schema::HistoricalInterpretationSensitivity,
    worth_relational::facade::schema::ProposedSchemaTransition,
    worth_relational::facade::schema::SchemaDiffAtom,
    worth_relational::facade::schema::SchemaDiffDetail,
    worth_relational::facade::schema::SchemaElementKind,
    worth_relational::facade::schema::SchemaElementRef, worth_relational::facade::schema::SchemaId,
    worth_relational::facade::schema::SchemaPublicationImpact,
    worth_relational::facade::schema::SchemaStratum,
    worth_relational::facade::schema::SchemaSubscriberImpact,
    worth_relational::facade::schema::SchemaVersionId,
    worth_relational::facade::schema::SubscriberBoundaryVisibility,
};

pub fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let mut baseline = crate::relational_certification::ordinary_context::world::supply_chain::scenarios::operating::build(scale);
    baseline.name = crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::VersionBoundary;
    baseline.pre_upgrade_schema = Some(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1);
    baseline.post_upgrade_schema = Some(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2);
    baseline
}

pub fn hazard_v2_transition() -> ProposedSchemaTransition {
    let schema_id = SchemaId("supply_chain".to_owned());
    ProposedSchemaTransition {
        source_schema_id: schema_id.clone(),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: schema_id.clone(),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![worth_relational::facade::schema::SchemaDiffAtom::new(
            worth_relational::facade::schema::SchemaElementRef::new(
                worth_relational::facade::schema::SchemaElementKind::EnumDomain,
                schema_id,
                SchemaVersionId(2),
                Some(entity_kind_id(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot)),
                "hazard",
            ),
            vec![
                worth_relational::facade::schema::SchemaStratum::ValueDomain,
                worth_relational::facade::schema::SchemaStratum::BehavioralSemantics,
                worth_relational::facade::schema::SchemaStratum::PublicationContract,
            ],
            worth_relational::facade::schema::SchemaPublicationImpact::ObservableSurfaceChanged,
            worth_relational::facade::schema::SchemaSubscriberImpact::ConsumableSurfaceChanged,
            worth_relational::facade::schema::HistoricalInterpretationSensitivity::SensitiveToValueMeaning,
            worth_relational::facade::schema::SchemaDiffDetail::EnumDomainExpanded {
                field: worth_foundational::facade::FieldKey::new("hazard").expect("canonical hazard field"),
                added_variants: vec!["HazardousV2".into()],
            },
        )
        .with_boundary_visibility_proof(
            worth_relational::facade::schema::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}
