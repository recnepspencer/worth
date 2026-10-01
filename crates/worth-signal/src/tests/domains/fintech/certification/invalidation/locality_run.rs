use crate::data::error::SignalError;
use crate::tests::domains::fintech::world::{
    ordinary_locality_cases, scheduled_locality_cases, FinancialLocalityScenario, LocalityLane,
};

use super::locality_completion::{certify_locality_completions, FinancialLocalityCompletions};
use super::{
    certify_current_strategy, certify_ordinary_cost_slopes_from_cases,
    certify_scheduled_cost_slopes_from_cases, FinancialCanonicalReportIdentity,
    FinancialLocalityCaseEvidence, InvalidationCostSlopeReport, InvalidationStrategyReport,
};

pub(in crate::tests::domains::fintech) struct FinancialFrontierLocalityCertificationRun {
    lane: LocalityLane,
    report_identity: FinancialCanonicalReportIdentity,
    cases: Vec<FinancialLocalityCaseEvidence>,
    slopes: [InvalidationCostSlopeReport; 3],
    strategy: InvalidationStrategyReport,
}

impl FinancialFrontierLocalityCertificationRun {
    pub(in crate::tests::domains::fintech) const fn lane(&self) -> LocalityLane {
        self.lane
    }

    pub(in crate::tests::domains::fintech) fn report_identity(
        &self,
    ) -> &FinancialCanonicalReportIdentity {
        &self.report_identity
    }

    pub(in crate::tests::domains::fintech) fn cases(&self) -> &[FinancialLocalityCaseEvidence] {
        &self.cases
    }

    pub(in crate::tests::domains::fintech) fn strategy(&self) -> &InvalidationStrategyReport {
        &self.strategy
    }

    pub(in crate::tests::domains::fintech) fn slopes(&self) -> &[InvalidationCostSlopeReport; 3] {
        &self.slopes
    }
}

pub(in crate::tests::domains::fintech) fn certify_ordinary_locality_run(
    seed: u64,
) -> Result<FinancialFrontierLocalityCertificationRun, SignalError> {
    let completions = certify_locality_completions(
        seed,
        LocalityLane::OrdinaryChangeGate,
        ordinary_locality_cases(),
    )?;
    seal_run(seed, LocalityLane::OrdinaryChangeGate, completions)
}

pub(in crate::tests::domains::fintech) fn certify_scheduled_locality_run(
    seed: u64,
) -> Result<FinancialFrontierLocalityCertificationRun, SignalError> {
    let completions =
        certify_locality_completions(seed, LocalityLane::Scheduled, scheduled_locality_cases())?;
    seal_run(seed, LocalityLane::Scheduled, completions)
}

fn seal_run(
    seed: u64,
    lane: LocalityLane,
    completions: FinancialLocalityCompletions,
) -> Result<FinancialFrontierLocalityCertificationRun, SignalError> {
    let FinancialLocalityCompletions {
        sparse,
        partitioned,
        convergent,
        dense,
        churn,
        restore,
    } = completions;
    let mut cases = Vec::new();
    cases.extend(sparse.into_cases());
    cases.extend(partitioned.into_cases());
    cases.extend(convergent.into_cases());
    cases.extend(dense.into_cases());
    cases.extend(churn.into_cases());
    cases.extend(restore.into_cases());
    require_complete_scenario_set(lane, &cases)?;
    require_unique_case_identities(&cases)?;
    let report_identity = FinancialCanonicalReportIdentity::from_cases(
        cases.iter().map(FinancialLocalityCaseEvidence::identity),
    )?;
    let slopes = match lane {
        LocalityLane::OrdinaryChangeGate => certify_ordinary_cost_slopes_from_cases(&cases)?,
        LocalityLane::Scheduled => certify_scheduled_cost_slopes_from_cases(&cases)?,
    };
    Ok(FinancialFrontierLocalityCertificationRun {
        lane,
        report_identity,
        cases,
        slopes,
        strategy: certify_current_strategy(seed)?,
    })
}

fn require_unique_case_identities(
    cases: &[FinancialLocalityCaseEvidence],
) -> Result<(), SignalError> {
    let mut identities = std::collections::BTreeMap::new();
    for case in cases {
        let description = (case.scenario(), case.scale(), case.measurement().seed());
        if let Some(previous) = identities.insert(*case.identity().digest_bytes(), description) {
            return Err(SignalError::invalid_input(format!(
                "financial locality cases share one canonical identity: {previous:?} and {description:?}"
            )));
        }
    }
    Ok(())
}

fn require_complete_scenario_set(
    lane: LocalityLane,
    cases: &[FinancialLocalityCaseEvidence],
) -> Result<(), SignalError> {
    for scenario in FinancialLocalityScenario::ALL {
        if !cases
            .iter()
            .any(|case| case.scenario() == scenario && case.lane() == lane)
        {
            return Err(SignalError::invalid_input(format!(
                "locality run is missing {scenario:?} in {lane:?}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::domains::fintech::certification::invalidation::TraversalStrategyDecision;

    #[test]
    fn ordinary_run_seals_all_six_scenario_families() {
        let run = certify_ordinary_locality_run(41).unwrap();
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

    #[test]
    #[ignore = "scheduled 10^3/10^4/10^5 scale courtroom"]
    fn scheduled_run_seals_all_declared_scale_contracts() {
        let run = certify_scheduled_locality_run(41).unwrap();
        assert_eq!(run.lane(), LocalityLane::Scheduled);
    }
}
