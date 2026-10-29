pub(crate) use crate::fast::relational_certification::world::supply_chain::*;

#[test]
fn cost_reports_are_bound_to_profile_and_seed() {
    let court = SupplyChainScale::court();
    let definition = SupplyChainWorldDefinition::operating(court).unwrap();
    let mut report = court.cost_report(SupplyChainCostInputs {
        baseline: BaselineName::Operating,
        schema: SchemaVersion::V1,
        setup_entities: definition.entities.len(),
        setup_relations: definition.relations.len(),
        delta_steps: 1,
        trace_steps: 1,
        oracle_steps: 1,
        observations: 1,
        cargo_lots: court.cargo_lots,
    });
    report.profile = ScaleName::Standard;
    assert!(matches!(
        court.enforce_budget(&report, BaselineName::Operating, SchemaVersion::V1),
        Err(CostBudgetError::ProfileMismatch { .. })
    ));
    report.profile = ScaleName::Court;
    report.seed += 1;
    assert!(matches!(
        court.enforce_budget(&report, BaselineName::Operating, SchemaVersion::V1),
        Err(CostBudgetError::SeedMismatch { .. })
    ));
    report.seed = court.seed;
    report.baseline = BaselineName::EmptyInstallation;
    assert!(matches!(
        court.enforce_budget(&report, BaselineName::Operating, SchemaVersion::V1),
        Err(CostBudgetError::BaselineMismatch { .. })
    ));
    report.baseline = BaselineName::Operating;
    report.schema = SchemaVersion::V2;
    assert!(matches!(
        court.enforce_budget(&report, BaselineName::Operating, SchemaVersion::V1),
        Err(CostBudgetError::SchemaMismatch { .. })
    ));
}

pub(crate) fn profiles() -> [SupplyChainScale; 3] {
    [
        SupplyChainScale::court(),
        SupplyChainScale::standard(),
        SupplyChainScale::scale(),
    ]
}

pub(crate) fn entity_kinds() -> [EntityKind; 8] {
    [
        EntityKind::Port,
        EntityKind::Terminal,
        EntityKind::Berth,
        EntityKind::Vessel,
        EntityKind::Voyage,
        EntityKind::PortCall,
        EntityKind::CargoLot,
        EntityKind::Inspection,
    ]
}

pub(crate) fn relation_kinds() -> [RelationKind; 10] {
    [
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
    ]
}

pub(crate) fn expected_region(scale: SupplyChainScale, ordinal: usize) -> Region {
    match scale.region_index(ordinal) {
        0 => Region::NorthReach,
        1 => Region::SouthReach,
        index => Region::Generated(index as u16),
    }
}

pub(crate) fn expected_entity_total(scale: SupplyChainScale) -> usize {
    entity_kinds()
        .into_iter()
        .map(|kind| scale.count_for(EntityKey::new(kind, 0)))
        .sum()
}

pub(crate) fn expected_relation_count(scale: SupplyChainScale, kind: RelationKind) -> usize {
    match kind {
        RelationKind::TerminalAtPort => scale.terminals,
        RelationKind::BerthAtTerminal => scale.berths,
        RelationKind::VesselAssignedToBerth => 1,
        RelationKind::VoyageUsesVessel => scale.voyages,
        RelationKind::VoyageHasCall | RelationKind::CallAtPort => scale.voyages * 3,
        RelationKind::CallPrecedes => scale.voyages * 2,
        RelationKind::CargoBookedOnVoyage => scale.cargo_lots.div_ceil(2),
        RelationKind::InspectionCoversVessel => scale.vessels,
        RelationKind::SharesPilotageZone => 2,
    }
}

pub(crate) fn expected_relation_total(scale: SupplyChainScale) -> usize {
    relation_kinds()
        .into_iter()
        .map(|kind| expected_relation_count(scale, kind))
        .sum()
}
