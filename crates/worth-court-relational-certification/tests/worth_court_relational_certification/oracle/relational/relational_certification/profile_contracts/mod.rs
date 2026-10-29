#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainCostInputs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition, crate::relational_certification::ordinary_context::profile_contracts::entity_kinds, crate::relational_certification::ordinary_context::profile_contracts::expected_entity_total, crate::relational_certification::ordinary_context::profile_contracts::expected_region, crate::relational_certification::ordinary_context::profile_contracts::expected_relation_count, crate::relational_certification::ordinary_context::profile_contracts::expected_relation_total, crate::relational_certification::ordinary_context::profile_contracts::profiles, crate::relational_certification::ordinary_context::profile_contracts::relation_kinds, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace};

#[test]
fn profiles_have_exact_density_and_named_budget_limits() {
    for scale in profiles() {
        let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).unwrap();
        for kind in entity_kinds() {
            let observed = definition
                .entities
                .values()
                .filter(|record| record.kind() == kind)
                .count();
            assert_eq!(observed, scale.count_for(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(kind, 0)));
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
                .map(|kind| scale.count_for(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(kind, 0)))
                .sum::<usize>()
        );
        for ordinal in 0..scale.ports {
            let key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, ordinal as u32);
            let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(port) = definition.entity(key).unwrap() else {
                panic!("port density must retain port records");
            };
            assert_eq!(port.region, expected_region(scale, ordinal));
        }
    }
}

#[test]
fn named_anchors_and_seeded_generation_are_stable_across_profiles() {
    let court = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let standard = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard()).unwrap();
    assert_ne!(
        court.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2)),
        standard.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2))
    );
    for scale in profiles() {
        let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).unwrap();
        let repeat = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).unwrap();
        assert_eq!(definition, repeat);
        let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(meridian) = definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity()).unwrap()
        else {
            panic!("Meridian must remain a port");
        };
        assert_eq!(meridian.name, "Meridian");
        let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(southpoint) =
            definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint.entity()).unwrap()
        else {
            panic!("Southpoint must remain a port");
        };
        assert_eq!(southpoint.name, "Southpoint");
        let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(cargo) =
            definition.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity()).unwrap()
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
        let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).unwrap();
        let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
        let child = baseline
            .branch
            .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
            .unwrap();
        let child = apply(&child, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap();
        let trace = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace::new(
            scale,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora],
        );
        let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&child);
        let observed = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
        assert_eq!(compare(&expected, &observed), Ok(()));
        let report = scale.cost_report(SupplyChainCostInputs {
            baseline: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
            schema: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            setup_entities: definition.entities.len(),
            setup_relations: definition.relations.len(),
            delta_steps: child.ancestry.accepted.len(),
            trace_steps: trace.step_count(),
            oracle_steps: child.ancestry.accepted.len() + 1,
            observations: 1,
            cargo_lots: definition
                .entities
                .values()
                .filter(|record| record.kind() == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot)
                .count(),
        });
        assert!(
            scale
                .enforce_budget(&report, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1)
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
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(scale).unwrap();
    let cargo_lots = definition
        .entities
        .values()
        .filter(|record| record.kind() == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot)
        .count();
    let valid = scale.cost_report(SupplyChainCostInputs {
        baseline: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
        schema: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        setup_entities: definition.entities.len(),
        setup_relations: definition.relations.len(),
        delta_steps: 1,
        trace_steps: 1,
        oracle_steps: 1,
        observations: 1,
        cargo_lots,
    });
    for dimension in [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::DeltaSteps,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::TraceSteps,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::Observations,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::CargoLots,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::SetupEntities,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::SetupRelations,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::OracleSteps,
    ] {
        let mut report = valid.clone();
        match dimension {
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::DeltaSteps => report.delta_steps = scale.budget.max_delta_steps + 1,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::TraceSteps => report.trace_steps = scale.budget.max_trace_steps + 1,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::Observations => report.observations = scale.budget.max_observations + 1,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::CargoLots => report.cargo_lots = scale.budget.max_cargo_lots + 1,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::SetupEntities => {
                report.setup_entities = scale.budget.max_setup_entities + 1
            }
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::SetupRelations => {
                report.setup_relations = scale.budget.max_setup_relations + 1
            }
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::OracleSteps => report.oracle_steps = scale.budget.max_oracle_steps + 1,
        }
        assert!(matches!(
            scale
                .enforce_budget(&report, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1)
                .unwrap_err(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::Exceeded { dimension: observed, .. } if observed == dimension
        ));
    }
}
