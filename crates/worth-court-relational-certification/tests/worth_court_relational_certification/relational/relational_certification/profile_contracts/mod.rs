#[allow(unused_imports)] use crate::relational_certification::ordinary_context::profile_contracts::*;
pub(crate) use worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainCostInputs;

#[test]
fn cost_reports_are_bound_to_profile_and_seed() {
    let court = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(court).unwrap();
    let mut report = court.cost_report(SupplyChainCostInputs {
        baseline: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating,
        schema: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        setup_entities: definition.entities.len(),
        setup_relations: definition.relations.len(),
        delta_steps: 1,
        trace_steps: 1,
        oracle_steps: 1,
        observations: 1,
        cargo_lots: court.cargo_lots,
    });
    report.profile = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Standard;
    assert!(matches!(
        court.enforce_budget(&report, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::ProfileMismatch { .. })
    ));
    report.profile = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Court;
    report.seed += 1;
    assert!(matches!(
        court.enforce_budget(&report, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::SeedMismatch { .. })
    ));
    report.seed = court.seed;
    report.baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::EmptyInstallation;
    assert!(matches!(
        court.enforce_budget(&report, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::BaselineMismatch { .. })
    ));
    report.baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating;
    report.schema = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2;
    assert!(matches!(
        court.enforce_budget(&report, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::SchemaMismatch { .. })
    ));
}
#[path = "budget_overages/mod.rs"]
pub(crate) mod budget_overages;
