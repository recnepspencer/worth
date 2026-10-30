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
