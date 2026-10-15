use super::{BaselineName, SupplyChainBaseline};
use crate::relational_certification::world::supply_chain::scale::SupplyChainScale;
use crate::relational_certification::world::supply_chain::schema::SchemaVersion;
use crate::relational_certification::world::supply_chain::{entity_kind_id, EntityKind};
use worth_foundational::facade::FieldKey;
use worth_relational::facade::schema::{
    HistoricalInterpretationSensitivity, ProposedSchemaTransition, SchemaDiffAtom,
    SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId, SchemaPublicationImpact,
    SchemaStratum, SchemaSubscriberImpact, SchemaVersionId, SubscriberBoundaryVisibility,
};

pub(super) fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let mut baseline = super::operating::build(scale);
    baseline.name = BaselineName::VersionBoundary;
    baseline.pre_upgrade_schema = Some(SchemaVersion::V1);
    baseline.post_upgrade_schema = Some(SchemaVersion::V2);
    baseline
}

pub(crate) fn hazard_v2_transition() -> ProposedSchemaTransition {
    let schema_id = SchemaId("supply_chain".to_owned());
    ProposedSchemaTransition {
        source_schema_id: schema_id.clone(),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: schema_id.clone(),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::EnumDomain,
                schema_id,
                SchemaVersionId(2),
                Some(entity_kind_id(EntityKind::CargoLot)),
                "hazard",
            ),
            vec![
                SchemaStratum::ValueDomain,
                SchemaStratum::BehavioralSemantics,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::SensitiveToValueMeaning,
            SchemaDiffDetail::EnumDomainExpanded {
                field: FieldKey::new("hazard").expect("canonical hazard field"),
                added_variants: vec!["HazardousV2".into()],
            },
        )
        .with_boundary_visibility_proof(
            SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}
