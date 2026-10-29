pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::endpoint_integrity, crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_aspects, crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_aspects, crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};
pub use {worth_relational::facade::config::CascadeDeletePolicy, worth_relational::facade::config::CrossContextPolicy};
pub use {worth_relational::facade::schema::EntityKindRegistration, worth_relational::facade::schema::KindAspectContractDeclarations, worth_relational::facade::schema::RelationKindRegistration, worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaRegistryError, worth_relational::facade::schema::SchemaVersionId};

pub fn schema_registry(
    schema_version_id: SchemaVersionId,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    schema_registry_with_port_contract(schema_version_id, false)
}

pub fn schema_registry_with_altered_port_contract(
    schema_version_id: SchemaVersionId,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    schema_registry_with_port_contract(schema_version_id, true)
}

pub fn schema_registry_with_port_contract(
    schema_version_id: SchemaVersionId,
    alter_port_contract: bool,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let mut registry = worth_relational::facade::schema::RelationalSchemaRegistry::new();
    for kind in [
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection,
    ] {
        let mut aspects = entity_aspects(kind);
        if alter_port_contract && kind == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port {
            aspects.pop();
        }
        registry = registry.register_entity_kind(EntityKindRegistration {
            kind_id: entity_kind_id(kind),
            kind_name: format!("supply_chain.entity.{kind:?}"),
            schema_id: SchemaId("supply_chain".to_owned()),
            schema_version_id,
            aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::new(aspects),
        })?;
    }
    for kind in [
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone,
    ] {
        registry = registry.register_relation_kind(RelationKindRegistration {
            kind_id: relation_kind_id(kind),
            kind_name: format!("supply_chain.relation.{kind:?}"),
            schema_id: SchemaId("supply_chain".to_owned()),
            schema_version_id,
            cross_context_policy: worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::new(relation_aspects()),
            relation_integrity: endpoint_integrity(kind),
        })?;
    }
    Ok(registry)
}