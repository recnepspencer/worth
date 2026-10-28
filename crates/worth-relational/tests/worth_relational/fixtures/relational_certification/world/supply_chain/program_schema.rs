pub(crate) use super::schema_vocabulary::{
    endpoint_integrity, entity_aspects, entity_kind_id, relation_aspects, relation_kind_id,
};
pub(crate) use super::semantic_key::{EntityKind, RelationKind};
pub(crate) use worth_relational::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
pub(crate) use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaRegistryError, SchemaVersionId,
};

pub(crate) fn schema_registry(
    schema_version_id: SchemaVersionId,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    schema_registry_with_port_contract(schema_version_id, false)
}

pub(crate) fn schema_registry_with_altered_port_contract(
    schema_version_id: SchemaVersionId,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    schema_registry_with_port_contract(schema_version_id, true)
}

pub(crate) fn schema_registry_with_port_contract(
    schema_version_id: SchemaVersionId,
    alter_port_contract: bool,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let mut registry = RelationalSchemaRegistry::new();
    for kind in [
        EntityKind::Port,
        EntityKind::Terminal,
        EntityKind::Berth,
        EntityKind::Vessel,
        EntityKind::Voyage,
        EntityKind::PortCall,
        EntityKind::CargoLot,
        EntityKind::Inspection,
    ] {
        let mut aspects = entity_aspects(kind);
        if alter_port_contract && kind == EntityKind::Port {
            aspects.pop();
        }
        registry = registry.register_entity_kind(EntityKindRegistration {
            kind_id: entity_kind_id(kind),
            kind_name: format!("supply_chain.entity.{kind:?}"),
            schema_id: SchemaId("supply_chain".to_owned()),
            schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(aspects),
        })?;
    }
    for kind in [
        RelationKind::TerminalAtPort,
        RelationKind::BerthAtTerminal,
        RelationKind::VesselAssignedToBerth,
        RelationKind::VoyageUsesVessel,
        RelationKind::VoyageHasCall,
        RelationKind::CallAtPort,
        RelationKind::CallPrecedes,
        RelationKind::CargoBookedOnVoyage,
        RelationKind::InspectionCoversVessel,
        RelationKind::SharesPilotageZone,
    ] {
        registry = registry.register_relation_kind(RelationKindRegistration {
            kind_id: relation_kind_id(kind),
            kind_name: format!("supply_chain.relation.{kind:?}"),
            schema_id: SchemaId("supply_chain".to_owned()),
            schema_version_id,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::new(relation_aspects()),
            relation_integrity: endpoint_integrity(kind),
        })?;
    }
    Ok(registry)
}
