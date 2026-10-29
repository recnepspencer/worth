pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::Region, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainCostInputs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

pub(crate) fn profiles() -> [SupplyChainScale; 3] {
    [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::scale(),
    ]
}

pub(crate) fn entity_kinds() -> [EntityKind; 8] {
    [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection,
    ]
}

pub(crate) fn relation_kinds() -> [RelationKind; 10] {
    [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone,
    ]
}

pub(crate) fn expected_region(scale: SupplyChainScale, ordinal: usize) -> Region {
    match scale.region_index(ordinal) {
        0 => worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::Region::NorthReach,
        1 => worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::Region::SouthReach,
        index => worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::Region::Generated(index as u16),
    }
}

pub(crate) fn expected_entity_total(scale: SupplyChainScale) -> usize {
    entity_kinds()
        .into_iter()
        .map(|kind| scale.count_for(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(kind, 0)))
        .sum()
}

pub(crate) fn expected_relation_count(scale: SupplyChainScale, kind: RelationKind) -> usize {
    match kind {
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort => scale.terminals,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal => scale.berths,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth => 1,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel => scale.voyages,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall | worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort => scale.voyages * 3,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes => scale.voyages * 2,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage => scale.cargo_lots.div_ceil(2),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel => scale.vessels,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone => 2,
    }
}

pub(crate) fn expected_relation_total(scale: SupplyChainScale) -> usize {
    relation_kinds()
        .into_iter()
        .map(|kind| expected_relation_count(scale, kind))
        .sum()
}