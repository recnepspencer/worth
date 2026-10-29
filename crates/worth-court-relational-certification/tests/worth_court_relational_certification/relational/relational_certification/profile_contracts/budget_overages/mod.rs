#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace::SemanticTrace, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainCostInputs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition, crate::relational_certification::ordinary_context::profile_contracts::entity_kinds, crate::relational_certification::ordinary_context::profile_contracts::expected_entity_total, crate::relational_certification::ordinary_context::profile_contracts::expected_region, crate::relational_certification::ordinary_context::profile_contracts::expected_relation_count, crate::relational_certification::ordinary_context::profile_contracts::expected_relation_total, crate::relational_certification::ordinary_context::profile_contracts::profiles, crate::relational_certification::ordinary_context::profile_contracts::relation_kinds, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::trace};

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
