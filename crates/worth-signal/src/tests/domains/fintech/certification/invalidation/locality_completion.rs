use crate::data::error::SignalError;
use crate::facade::DiagnosticsTier;
use crate::tests::domains::fintech::world::{
    compile_financial_locality_world_at_tier, FinancialLocalityScenario,
    FinancialRestoreLifecycleEvidence, FinancialWorldDefinition, LocalityCaseContract,
    LocalityLane, LocalityScaleTuple,
};

use super::{verify_locality_case, FinancialLocalityCaseEvidence};

macro_rules! scenario_completion {
    ($name:ident) => {
        pub(in crate::tests::domains::fintech) struct $name {
            cases: Vec<FinancialLocalityCaseEvidence>,
        }

        impl $name {
            pub(super) fn into_cases(self) -> Vec<FinancialLocalityCaseEvidence> {
                self.cases
            }
        }
    };
}

scenario_completion!(SparseBookFanoutCompletion);
scenario_completion!(PartitionedCurveUniverseCompletion);
scenario_completion!(ConvergentFactorBatchCompletion);
scenario_completion!(DenseMarketCloseCompletion);
scenario_completion!(PortfolioDependencyChurnCompletion);

pub(in crate::tests::domains::fintech) struct BranchRestoreLocalityReplayCompletion {
    cases: Vec<FinancialLocalityCaseEvidence>,
    _lifecycle: Vec<FinancialRestoreLifecycleEvidence>,
}

impl BranchRestoreLocalityReplayCompletion {
    pub(super) fn into_cases(self) -> Vec<FinancialLocalityCaseEvidence> {
        self.cases
    }
}

pub(super) struct FinancialLocalityCompletions {
    pub(super) sparse: SparseBookFanoutCompletion,
    pub(super) partitioned: PartitionedCurveUniverseCompletion,
    pub(super) convergent: ConvergentFactorBatchCompletion,
    pub(super) dense: DenseMarketCloseCompletion,
    pub(super) churn: PortfolioDependencyChurnCompletion,
    pub(super) restore: BranchRestoreLocalityReplayCompletion,
}

pub(super) fn certify_locality_completions(
    seed: u64,
    lane: LocalityLane,
    cases: impl IntoIterator<Item = LocalityCaseContract>,
) -> Result<FinancialLocalityCompletions, SignalError> {
    let cases = cases.into_iter().collect::<Vec<_>>();
    let sparse = SparseBookFanoutCompletion {
        cases: certify_family(
            seed,
            lane,
            &cases,
            FinancialLocalityScenario::SparseBookFanout,
        )?,
    };
    let partitioned = PartitionedCurveUniverseCompletion {
        cases: certify_family(
            seed,
            lane,
            &cases,
            FinancialLocalityScenario::PartitionedCurveUniverse,
        )?,
    };
    let convergent = ConvergentFactorBatchCompletion {
        cases: certify_family(
            seed,
            lane,
            &cases,
            FinancialLocalityScenario::ConvergentFactorBatch,
        )?,
    };
    let dense = DenseMarketCloseCompletion {
        cases: certify_family(
            seed,
            lane,
            &cases,
            FinancialLocalityScenario::DenseMarketClose,
        )?,
    };
    let churn = PortfolioDependencyChurnCompletion {
        cases: certify_family(
            seed,
            lane,
            &cases,
            FinancialLocalityScenario::PortfolioDependencyChurn,
        )?,
    };
    let restore_cases = family_contracts(
        lane,
        &cases,
        FinancialLocalityScenario::BranchRestoreLocalityReplay,
    )?;
    let mut restore_evidence = Vec::new();
    let mut restore_lifecycle = Vec::new();
    for case in restore_cases {
        for case_seed in declared_case_seeds(seed, case.scale) {
            let definition = FinancialWorldDefinition::locality_case(case_seed, case);
            let trace_count = definition.locality().unwrap().action_traces().len();
            for trace_index in 0..trace_count {
                report_scheduled_case(lane, case, case_seed, trace_index);
                restore_evidence.push(verify_locality_case(
                    FinancialWorldDefinition::locality_case(case_seed, case),
                    trace_index,
                    DiagnosticsTier::Operational,
                    1,
                )
                .inspect_err(|error| {
                    eprintln!(
                        "locality completion failed: lane={lane:?} scenario={:?} scale={:?} seed={case_seed} trace={trace_index}: {error:?}",
                        case.scenario(), case.scale
                    );
                })?);
            }
            let mut compiled = compile_financial_locality_world_at_tier(
                definition,
                DiagnosticsTier::Operational,
            )
            .inspect_err(|error| {
                eprintln!(
                    "locality restore compile failed: lane={lane:?} scale={:?} seed={case_seed}: {error:?}",
                    case.scale
                );
            })?;
            restore_lifecycle.push(compiled.certify_restore_locality_lifecycle().inspect_err(
                |error| {
                    eprintln!(
                        "locality restore lifecycle failed: lane={lane:?} scale={:?} seed={case_seed}: {error:?}",
                        case.scale
                    );
                },
            )?);
        }
    }
    Ok(FinancialLocalityCompletions {
        sparse,
        partitioned,
        convergent,
        dense,
        churn,
        restore: BranchRestoreLocalityReplayCompletion {
            cases: restore_evidence,
            _lifecycle: restore_lifecycle,
        },
    })
}

fn certify_family(
    seed: u64,
    lane: LocalityLane,
    cases: &[LocalityCaseContract],
    scenario: FinancialLocalityScenario,
) -> Result<Vec<FinancialLocalityCaseEvidence>, SignalError> {
    let mut evidence = Vec::new();
    for case in family_contracts(lane, cases, scenario)? {
        for case_seed in declared_case_seeds(seed, case.scale) {
            let definition = FinancialWorldDefinition::locality_case(case_seed, case);
            let trace_count = definition.locality().unwrap().action_traces().len();
            for trace_index in 0..trace_count {
                report_scheduled_case(lane, case, case_seed, trace_index);
                evidence.push(verify_locality_case(
                    FinancialWorldDefinition::locality_case(case_seed, case),
                    trace_index,
                    DiagnosticsTier::Operational,
                    1,
                )
                .inspect_err(|error| {
                    eprintln!(
                        "locality completion failed: lane={lane:?} scenario={scenario:?} scale={:?} seed={case_seed} trace={trace_index}: {error:?}",
                        case.scale
                    );
                })?);
            }
        }
    }
    Ok(evidence)
}

fn report_scheduled_case(
    lane: LocalityLane,
    case: LocalityCaseContract,
    seed: u64,
    trace_index: usize,
) {
    if lane == LocalityLane::Scheduled {
        eprintln!(
            "M13 scheduled evidence: {:?} seed={seed} trace={trace_index}",
            case.scale
        );
    }
}

fn declared_case_seeds(seed: u64, scale: LocalityScaleTuple) -> impl Iterator<Item = u64> {
    let count = match scale {
        LocalityScaleTuple::ConvergentFactorBatch {
            canonical_seeds, ..
        }
        | LocalityScaleTuple::PortfolioDependencyChurn {
            canonical_seeds, ..
        }
        | LocalityScaleTuple::BranchRestoreLocalityReplay {
            canonical_seeds, ..
        } => canonical_seeds,
        LocalityScaleTuple::SparseBookFanout { .. }
        | LocalityScaleTuple::PartitionedCurveUniverse { .. }
        | LocalityScaleTuple::DenseMarketClose { .. } => 1,
    };
    (0..count).map(move |ordinal| seed.saturating_add(u64::from(ordinal)))
}

fn family_contracts(
    lane: LocalityLane,
    cases: &[LocalityCaseContract],
    scenario: FinancialLocalityScenario,
) -> Result<Vec<LocalityCaseContract>, SignalError> {
    let family = cases
        .iter()
        .copied()
        .filter(|case| case.scenario() == scenario && case.lane == lane)
        .collect::<Vec<_>>();
    if family.is_empty() {
        Err(SignalError::invalid_input(format!(
            "locality completion is missing {scenario:?} in {lane:?}"
        )))
    } else {
        Ok(family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_seed_counts_drive_the_scheduled_case_runner() {
        let convergent = LocalityScaleTuple::ConvergentFactorBatch {
            producer_permutations: 24,
            duplicate_admissions: 0,
            canonical_seeds: 16,
        };
        let restore = LocalityScaleTuple::BranchRestoreLocalityReplay {
            posture: crate::tests::domains::fintech::world::RestorePosture::Narrow,
            total_outputs: 0,
            canonical_seeds: 8,
        };

        assert_eq!(
            declared_case_seeds(41, convergent).collect::<Vec<_>>(),
            (41..57).collect::<Vec<_>>()
        );
        assert_eq!(
            declared_case_seeds(41, restore).collect::<Vec<_>>(),
            (41..49).collect::<Vec<_>>()
        );
    }
}
