mod artifact_materialization;
mod aspects;
mod audit_surface;
mod branch_checkpoint;
mod branch_history;
mod branch_isolation;
mod certification;
mod certification_naming;
mod evaluation;
mod execution_tier;
mod executor_overlap;
mod fanout_tolerance;
mod fixture;
mod hierarchy;
mod invalidation;
mod keyed_cache;
mod market_seed;
mod market_state;
mod node_families;
mod partition_locality;
mod partition_surface;
mod recovery_errors;
mod regimes;
mod scales;
mod scenarios;
mod snapshot_recovery;
mod threshold_flapping;
mod tier_policy;
mod truth_comparison;
mod truth_snapshot;
mod world;
mod world_handles;
mod world_setup;
mod world_shape;

pub(crate) use certification::invalidation::verify_locality_case_with_policy;
pub(crate) use certification::invalidation::{verify_locality_case, FreshFinancialRecompute};
pub(crate) use regimes::MarketRegime;
pub(crate) use scales::FintechScale;
pub(crate) use scenarios::setup_seeded_world_with;
pub(crate) use world::FinancialWorldDefinition;
pub(crate) use world::{
    compile_financial_locality_world_with_policy, compile_financial_world_with_policy,
    CompiledFinancialWorld, DensityRatio, FinancialPerformanceBatchReport,
    LocalityOptionalObservationInventory,
};

pub(crate) fn restore_lifecycle_definition(seed: u64) -> FinancialWorldDefinition {
    let case = world::ordinary_locality_cases()
        .into_iter()
        .find(|case| {
            case.scenario() == world::FinancialLocalityScenario::BranchRestoreLocalityReplay
        })
        .expect("restore lifecycle case is registered");
    FinancialWorldDefinition::locality_case(seed, case)
}
