use crate::data::error::SignalError;
use crate::data::telemetry::InvalidationPerformedCounter;

use super::{
    verify_locality_case, ExpectedLocalityCounterRow, FinancialCanonicalCaseIdentity,
    FinancialLocalityCaseEvidence,
};
use crate::tests::domains::fintech::world::strategy_work_projection;
use crate::tests::domains::fintech::world::{FinancialLocalityScenario, FinancialWorldDefinition};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) enum TraversalStrategyDecision {
    CurrentStrategyCertified,
}

#[derive(Debug)]
pub(in crate::tests::domains::fintech) struct InvalidationStrategyReport {
    decision: TraversalStrategyDecision,
    deterministic_case: Option<FinancialCanonicalCaseIdentity>,
    optimized_case: Option<FinancialCanonicalCaseIdentity>,
    canonical_work_items: u64,
}

impl InvalidationStrategyReport {
    pub(in crate::tests::domains::fintech) const fn decision(&self) -> TraversalStrategyDecision {
        self.decision
    }

    pub(in crate::tests::domains::fintech) const fn canonical_work_items(&self) -> u64 {
        self.canonical_work_items
    }

    pub(in crate::tests::domains::fintech) fn case_identities(
        &self,
    ) -> Option<(
        &FinancialCanonicalCaseIdentity,
        &FinancialCanonicalCaseIdentity,
    )> {
        self.deterministic_case
            .as_ref()
            .zip(self.optimized_case.as_ref())
    }
}

pub(in crate::tests::domains::fintech) fn certify_current_strategy(
    seed: u64,
) -> Result<InvalidationStrategyReport, SignalError> {
    let definition = || {
        FinancialWorldDefinition::dense_market_close(
            seed,
            1_000,
            crate::tests::domains::fintech::world::DensityRatio::FourInFive,
        )
    };
    let deterministic = verify_locality_case(
        definition(),
        0,
        crate::facade::DiagnosticsTier::Operational,
        1,
    )?;
    let optimized = verify_locality_case(
        definition(),
        0,
        crate::facade::DiagnosticsTier::Operational,
        4,
    )?;
    certify_equivalent_streams(deterministic, optimized)
}

fn certify_equivalent_streams(
    deterministic: FinancialLocalityCaseEvidence,
    optimized: FinancialLocalityCaseEvidence,
) -> Result<InvalidationStrategyReport, SignalError> {
    if deterministic.scenario() != FinancialLocalityScenario::DenseMarketClose
        || deterministic.scenario() != optimized.scenario()
        || deterministic.scale() != optimized.scale()
    {
        return Err(SignalError::internal(
            "strategy comparison mixed financial case identity",
        ));
    }
    if strategy_work_projection(deterministic.performed_work())
        != strategy_work_projection(optimized.performed_work())
    {
        return Err(SignalError::internal(
            "strategies performed different canonical admitted work",
        ));
    }
    if !ExpectedLocalityCounterRow::ALL
        .into_iter()
        .zip(InvalidationPerformedCounter::ALL)
        .filter(|(row, _)| !row.is_physical_batch_shape())
        .all(|(_, counter)| {
            deterministic.counters().value(counter) == optimized.counters().value(counter)
        })
        || deterministic.necessary_evaluations() != optimized.necessary_evaluations()
        || deterministic.identity() != optimized.identity()
    {
        return Err(SignalError::internal(
            "strategies committed different performed truth or evidence",
        ));
    }
    let canonical_work_items = deterministic.performed_work().len() as u64;
    Ok(InvalidationStrategyReport {
        decision: TraversalStrategyDecision::CurrentStrategyCertified,
        deterministic_case: Some(deterministic.identity().clone()),
        optimized_case: Some(optimized.identity().clone()),
        canonical_work_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_optimized_modes_consume_the_same_canonical_work() {
        let report = certify_current_strategy(41).unwrap();
        assert_eq!(
            report.decision(),
            TraversalStrategyDecision::CurrentStrategyCertified
        );
        assert!(report.canonical_work_items() > 0);
        let (deterministic, optimized) = report.case_identities().unwrap();
        assert_eq!(deterministic, optimized);
    }

    #[test]
    fn strategy_report_rejects_a_mixed_financial_work_stream() {
        let deterministic = verify_locality_case(
            FinancialWorldDefinition::dense_market_close(
                41,
                1_000,
                crate::tests::domains::fintech::world::DensityRatio::OneInOneHundred,
            ),
            0,
            crate::facade::DiagnosticsTier::Operational,
            1,
        )
        .unwrap();
        let optimized = verify_locality_case(
            FinancialWorldDefinition::dense_market_close(
                41,
                1_000,
                crate::tests::domains::fintech::world::DensityRatio::FourInFive,
            ),
            0,
            crate::facade::DiagnosticsTier::Operational,
            4,
        )
        .unwrap();

        assert!(certify_equivalent_streams(deterministic, optimized).is_err());
    }
}
