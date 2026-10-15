use crate::relational_certification::world::supply_chain::*;

#[test]
fn profiles_have_exact_density_and_named_budget_limits() {
    for scale in profiles() {
        let definition = SupplyChainWorldDefinition::operating(scale).unwrap();
        for kind in entity_kinds() {
            let observed = definition
                .entities
                .values()
                .filter(|record| record.kind() == kind)
                .count();
            assert_eq!(observed, scale.count_for(EntityKey::new(kind, 0)));
        }
        for kind in relation_kinds() {
            let observed = definition
                .relations
                .values()
                .filter(|edge| edge.key.kind == kind)
                .count();
            assert_eq!(observed, expected_relation_count(scale, kind));
        }
        assert_eq!(definition.entities.len(), expected_entity_total(scale));
        assert_eq!(definition.relations.len(), expected_relation_total(scale));
        assert_eq!(
            definition.entities.len(),
            entity_kinds()
                .into_iter()
                .map(|kind| scale.count_for(EntityKey::new(kind, 0)))
                .sum::<usize>()
        );
        for ordinal in 0..scale.ports {
            let key = EntityKey::new(EntityKind::Port, ordinal as u32);
            let EntityRecord::Port(port) = definition.entity(key).unwrap() else {
                panic!("port density must retain port records");
            };
            assert_eq!(port.region, expected_region(scale, ordinal));
        }
    }
}

#[test]
fn named_anchors_and_seeded_generation_are_stable_across_profiles() {
    let court = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let standard = SupplyChainWorldDefinition::operating(SupplyChainScale::standard()).unwrap();
    assert_ne!(
        court.entity(EntityKey::new(EntityKind::Port, 2)),
        standard.entity(EntityKey::new(EntityKind::Port, 2))
    );
    for scale in profiles() {
        let definition = SupplyChainWorldDefinition::operating(scale).unwrap();
        let repeat = SupplyChainWorldDefinition::operating(scale).unwrap();
        assert_eq!(definition, repeat);
        let EntityRecord::Port(meridian) = definition.entity(Anchor::Meridian.entity()).unwrap()
        else {
            panic!("Meridian must remain a port");
        };
        assert_eq!(meridian.name, "Meridian");
        let EntityRecord::Port(southpoint) =
            definition.entity(Anchor::Southpoint.entity()).unwrap()
        else {
            panic!("Southpoint must remain a port");
        };
        assert_eq!(southpoint.name, "Southpoint");
        let EntityRecord::CargoLot(cargo) =
            definition.entity(Anchor::MedicalSupplies.entity()).unwrap()
        else {
            panic!("Medical Supplies must remain cargo");
        };
        assert_eq!(cargo.customer_code.0, "CARGO-MEDICAL-0000");
        for anchor in scale.anchors() {
            assert!(
                definition.entity(anchor.entity()).is_some(),
                "missing {anchor}"
            );
        }
    }
}

#[test]
fn all_profiles_construct_with_seeded_regions_and_enforced_cost_report() {
    for scale in profiles() {
        let definition = SupplyChainWorldDefinition::operating(scale).unwrap();
        let baseline = SupplyChainBaseline::operating(scale);
        let child = baseline
            .branch
            .fork(BranchLabel::Storm, BranchLabel::Operating)
            .unwrap();
        let child = apply(&child, DeltaId::StormRerouteAurora).unwrap();
        let trace = SemanticTrace::new(
            scale,
            BaselineName::Operating,
            BranchLabel::Storm,
            vec![DeltaId::StormRerouteAurora],
        );
        let expected = ExpectedSupplyChainObservation::from_branch(&child);
        let observed = ObservedSupplyChainState::from_expected(&expected);
        assert_eq!(compare(&expected, &observed), Ok(()));
        let report = scale.cost_report(SupplyChainCostInputs {
            baseline: BaselineName::Operating,
            schema: SchemaVersion::V1,
            setup_entities: definition.entities.len(),
            setup_relations: definition.relations.len(),
            delta_steps: child.ancestry.accepted.len(),
            trace_steps: trace.step_count(),
            oracle_steps: child.ancestry.accepted.len() + 1,
            observations: 1,
            cargo_lots: definition
                .entities
                .values()
                .filter(|record| record.kind() == EntityKind::CargoLot)
                .count(),
        });
        assert!(
            scale
                .enforce_budget(&report, BaselineName::Operating, SchemaVersion::V1)
                .is_ok(),
            "{report:?}"
        );
        let machine = report.machine_report();
        for field in [
            "profile=",
            "seed=",
            "baseline=",
            "schema=",
            "setup_entities=",
            "setup_relations=",
            "delta_steps=",
            "trace_steps=",
            "oracle_steps=",
            "observations=",
            "cargo_lots=",
        ] {
            assert!(machine.contains(field), "missing {field} in {machine}");
        }
    }
}

#[test]
fn every_declared_budget_dimension_rejects_an_independent_overage() {
    let scale = SupplyChainScale::court();
    let definition = SupplyChainWorldDefinition::operating(scale).unwrap();
    let cargo_lots = definition
        .entities
        .values()
        .filter(|record| record.kind() == EntityKind::CargoLot)
        .count();
    let valid = scale.cost_report(SupplyChainCostInputs {
        baseline: BaselineName::Operating,
        schema: SchemaVersion::V1,
        setup_entities: definition.entities.len(),
        setup_relations: definition.relations.len(),
        delta_steps: 1,
        trace_steps: 1,
        oracle_steps: 1,
        observations: 1,
        cargo_lots,
    });
    for dimension in [
        CostDimension::DeltaSteps,
        CostDimension::TraceSteps,
        CostDimension::Observations,
        CostDimension::CargoLots,
        CostDimension::SetupEntities,
        CostDimension::SetupRelations,
        CostDimension::OracleSteps,
    ] {
        let mut report = valid.clone();
        match dimension {
            CostDimension::DeltaSteps => report.delta_steps = scale.budget.max_delta_steps + 1,
            CostDimension::TraceSteps => report.trace_steps = scale.budget.max_trace_steps + 1,
            CostDimension::Observations => report.observations = scale.budget.max_observations + 1,
            CostDimension::CargoLots => report.cargo_lots = scale.budget.max_cargo_lots + 1,
            CostDimension::SetupEntities => {
                report.setup_entities = scale.budget.max_setup_entities + 1
            }
            CostDimension::SetupRelations => {
                report.setup_relations = scale.budget.max_setup_relations + 1
            }
            CostDimension::OracleSteps => report.oracle_steps = scale.budget.max_oracle_steps + 1,
        }
        assert!(matches!(
            scale
                .enforce_budget(&report, BaselineName::Operating, SchemaVersion::V1)
                .unwrap_err(),
            CostBudgetError::Exceeded { dimension: observed, .. } if observed == dimension
        ));
    }
}

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

fn profiles() -> [SupplyChainScale; 3] {
    [
        SupplyChainScale::court(),
        SupplyChainScale::standard(),
        SupplyChainScale::scale(),
    ]
}

fn entity_kinds() -> [EntityKind; 8] {
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

fn relation_kinds() -> [RelationKind; 10] {
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

fn expected_region(scale: SupplyChainScale, ordinal: usize) -> Region {
    match scale.region_index(ordinal) {
        0 => Region::NorthReach,
        1 => Region::SouthReach,
        index => Region::Generated(index as u16),
    }
}

fn expected_entity_total(scale: SupplyChainScale) -> usize {
    entity_kinds()
        .into_iter()
        .map(|kind| scale.count_for(EntityKey::new(kind, 0)))
        .sum()
}

fn expected_relation_count(scale: SupplyChainScale, kind: RelationKind) -> usize {
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

fn expected_relation_total(scale: SupplyChainScale) -> usize {
    relation_kinds()
        .into_iter()
        .map(|kind| expected_relation_count(scale, kind))
        .sum()
}
