use std::sync::OnceLock;

use crate::tests::domains::fintech::world::LocalityScaleTuple;

use super::super::locality_completion::{
    certify_family, certify_restore_family, BranchRestoreLocalityReplayCompletion,
};
use super::super::TraversalStrategyDecision;
use super::*;

#[path = "ordinary/sparse.rs"]
mod sparse;

fn family_evidence(
    scenario: FinancialLocalityScenario,
) -> &'static [FinancialLocalityCaseEvidence] {
    static SPARSE: OnceLock<Vec<FinancialLocalityCaseEvidence>> = OnceLock::new();
    static PARTITIONED: OnceLock<Vec<FinancialLocalityCaseEvidence>> = OnceLock::new();
    static CONVERGENT: OnceLock<Vec<FinancialLocalityCaseEvidence>> = OnceLock::new();
    static DENSE: OnceLock<Vec<FinancialLocalityCaseEvidence>> = OnceLock::new();
    static CHURN: OnceLock<Vec<FinancialLocalityCaseEvidence>> = OnceLock::new();
    static RESTORE: OnceLock<BranchRestoreLocalityReplayCompletion> = OnceLock::new();
    let cases = ordinary_locality_cases();
    let cache = match scenario {
        FinancialLocalityScenario::SparseBookFanout => {
            return SPARSE.get_or_init(|| {
                cases
                    .iter()
                    .filter_map(|case| match case.scale {
                        LocalityScaleTuple::SparseBookFanout {
                            total_outputs,
                            axis,
                        } => Some(
                            sparse::contract_evidence(total_outputs, axis)
                                .iter()
                                .cloned(),
                        ),
                        _ => None,
                    })
                    .flatten()
                    .collect()
            });
        }
        FinancialLocalityScenario::PartitionedCurveUniverse => &PARTITIONED,
        FinancialLocalityScenario::ConvergentFactorBatch => &CONVERGENT,
        FinancialLocalityScenario::DenseMarketClose => &DENSE,
        FinancialLocalityScenario::PortfolioDependencyChurn => &CHURN,
        FinancialLocalityScenario::BranchRestoreLocalityReplay => {
            return RESTORE
                .get_or_init(|| {
                    certify_restore_family(41, LocalityLane::OrdinaryChangeGate, &cases).unwrap()
                })
                .cases();
        }
    };
    cache.get_or_init(|| {
        certify_family(41, LocalityLane::OrdinaryChangeGate, &cases, scenario).unwrap()
    })
}

fn assert_ordinary_family(scenario: FinancialLocalityScenario) {
    assert_ordinary_cases(scenario, family_evidence(scenario));
}

fn assert_ordinary_cases(
    scenario: FinancialLocalityScenario,
    cases: &[FinancialLocalityCaseEvidence],
) {
    assert!(!cases.is_empty());
    assert!(cases.iter().all(|case| {
        let measurement = case.measurement();
        case.scenario() == scenario
            && case.lane() == LocalityLane::OrdinaryChangeGate
            && measurement.seed() == 41
            && !measurement.elapsed().is_zero()
            && measurement.peak_batch_memory_items() > 0
    }));
}

#[test]
fn ordinary_sparse_book_fanout_completes() {
    assert_ordinary_family(FinancialLocalityScenario::SparseBookFanout);
}

#[test]
fn ordinary_partitioned_curve_universe_completes() {
    assert_ordinary_family(FinancialLocalityScenario::PartitionedCurveUniverse);
}

#[test]
fn ordinary_convergent_factor_batch_completes() {
    assert_ordinary_family(FinancialLocalityScenario::ConvergentFactorBatch);
}

#[test]
fn ordinary_dense_market_close_completes() {
    assert_ordinary_family(FinancialLocalityScenario::DenseMarketClose);
}

#[test]
fn ordinary_portfolio_dependency_churn_completes() {
    assert_ordinary_family(FinancialLocalityScenario::PortfolioDependencyChurn);
}

#[test]
fn ordinary_branch_restore_locality_replay_completes() {
    assert_ordinary_family(FinancialLocalityScenario::BranchRestoreLocalityReplay);
}

#[test]
fn ordinary_run_seals_all_six_scenario_families() {
    let cases = FinancialLocalityScenario::ALL
        .into_iter()
        .flat_map(|scenario| family_evidence(scenario).iter().cloned())
        .collect();
    let run = certify_ordinary_locality_run(41, cases).unwrap();
    assert_eq!(run.lane(), LocalityLane::OrdinaryChangeGate);
    assert_ne!(run.report_identity().digest_bytes(), &[0; 32]);
    assert_eq!(run.slopes().len(), 3);
    assert!(run.cases().iter().all(|case| {
        let measurement = case.measurement();
        measurement.seed() == 41
            && !measurement.elapsed().is_zero()
            && measurement.peak_batch_memory_items() > 0
    }));
    for scenario in FinancialLocalityScenario::ALL {
        assert!(run.cases().iter().any(|case| case.scenario() == scenario));
    }
    assert_eq!(
        run.strategy().decision(),
        TraversalStrategyDecision::CurrentStrategyCertified
    );
}
