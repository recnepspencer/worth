use crate::data::error::SignalError;
use crate::data::telemetry::InvalidationPerformedCounter as Counter;
use crate::facade::DiagnosticsTier;
use crate::tests::domains::fintech::world::{
    FinancialLocalityScenario, FinancialWorldDefinition, LocalityScaleTuple, SparseFanoutAxis,
};

use super::{verify_locality_case, FinancialCanonicalCaseIdentity, FinancialLocalityCaseEvidence};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::tests::domains::fintech) enum InvalidationCostSlopeClass {
    IndexDisjoint,
    QueriedCandidate,
    SemanticFrontier,
}

#[derive(Debug)]
pub(in crate::tests::domains::fintech) struct InvalidationCostSlopeReport {
    class: InvalidationCostSlopeClass,
    lower_case: FinancialCanonicalCaseIdentity,
    upper_case: FinancialCanonicalCaseIdentity,
    added_candidates: u64,
    added_semantic_work: u64,
}

impl InvalidationCostSlopeReport {
    pub(in crate::tests::domains::fintech) const fn class(&self) -> InvalidationCostSlopeClass {
        self.class
    }

    pub(in crate::tests::domains::fintech) const fn added_candidates(&self) -> u64 {
        self.added_candidates
    }

    pub(in crate::tests::domains::fintech) const fn added_semantic_work(&self) -> u64 {
        self.added_semantic_work
    }

    pub(in crate::tests::domains::fintech) fn case_identities(
        &self,
    ) -> (
        &FinancialCanonicalCaseIdentity,
        &FinancialCanonicalCaseIdentity,
    ) {
        (&self.lower_case, &self.upper_case)
    }
}

pub(in crate::tests::domains::fintech) fn certify_ordinary_cost_slopes(
    seed: u64,
) -> Result<[InvalidationCostSlopeReport; 3], SignalError> {
    let index_lower = run(FinancialWorldDefinition::sparse_book_fanout(
        seed,
        64,
        SparseFanoutAxis::IndexDisjoint,
    ))?;
    let index_upper = run(FinancialWorldDefinition::sparse_book_fanout(
        seed,
        512,
        SparseFanoutAxis::IndexDisjoint,
    ))?;
    let queried_lower = run(FinancialWorldDefinition::sparse_book_fanout(
        seed,
        64,
        SparseFanoutAxis::QueriedRejecting,
    ))?;
    let queried_upper = run(FinancialWorldDefinition::sparse_book_fanout(
        seed,
        512,
        SparseFanoutAxis::QueriedRejecting,
    ))?;
    let semantic_lower = run(FinancialWorldDefinition::partitioned_curve_universe(
        seed, 16, 1, 1,
    ))?;
    let semantic_upper = run(FinancialWorldDefinition::partitioned_curve_universe(
        seed, 16, 1, 8,
    ))?;
    Ok([
        certify_index_disjoint(index_lower, index_upper)?,
        certify_queried_candidate(queried_lower, queried_upper, 448)?,
        certify_semantic_frontier(semantic_lower, semantic_upper, 7)?,
    ])
}

pub(in crate::tests::domains::fintech) fn certify_ordinary_cost_slopes_from_cases(
    cases: &[FinancialLocalityCaseEvidence],
) -> Result<[InvalidationCostSlopeReport; 3], SignalError> {
    let find = |scale| {
        cases
            .iter()
            .find(|case| case.scale() == scale)
            .cloned()
            .ok_or_else(|| SignalError::invalid_input("ordinary slope evidence is incomplete"))
    };
    Ok([
        certify_index_disjoint(
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 64,
                axis: SparseFanoutAxis::IndexDisjoint,
            })?,
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 512,
                axis: SparseFanoutAxis::IndexDisjoint,
            })?,
        )?,
        certify_queried_candidate(
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 64,
                axis: SparseFanoutAxis::QueriedRejecting,
            })?,
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 512,
                axis: SparseFanoutAxis::QueriedRejecting,
            })?,
            448,
        )?,
        certify_semantic_frontier(
            find(LocalityScaleTuple::PartitionedCurveUniverse {
                regions: 16,
                matching_memberships: 1,
                instruments_per_matching_region: 1,
            })?,
            find(LocalityScaleTuple::PartitionedCurveUniverse {
                regions: 16,
                matching_memberships: 1,
                instruments_per_matching_region: 8,
            })?,
            7,
        )?,
    ])
}

pub(in crate::tests::domains::fintech) fn certify_scheduled_cost_slopes_from_cases(
    cases: &[FinancialLocalityCaseEvidence],
) -> Result<[InvalidationCostSlopeReport; 3], SignalError> {
    let find = |scale| {
        cases
            .iter()
            .find(|case| case.scale() == scale)
            .cloned()
            .ok_or_else(|| SignalError::invalid_input("scheduled slope evidence is incomplete"))
    };
    Ok([
        certify_index_disjoint(
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 1_000,
                axis: SparseFanoutAxis::IndexDisjoint,
            })?,
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 10_000,
                axis: SparseFanoutAxis::IndexDisjoint,
            })?,
        )?,
        certify_queried_candidate(
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 1_000,
                axis: SparseFanoutAxis::QueriedRejecting,
            })?,
            find(LocalityScaleTuple::SparseBookFanout {
                total_outputs: 10_000,
                axis: SparseFanoutAxis::QueriedRejecting,
            })?,
            9_000,
        )?,
        certify_semantic_frontier(
            find(LocalityScaleTuple::PartitionedCurveUniverse {
                regions: 1_024,
                matching_memberships: 1,
                instruments_per_matching_region: 1,
            })?,
            find(LocalityScaleTuple::PartitionedCurveUniverse {
                regions: 1_024,
                matching_memberships: 1,
                instruments_per_matching_region: 32,
            })?,
            31,
        )?,
    ])
}

fn run(definition: FinancialWorldDefinition) -> Result<FinancialLocalityCaseEvidence, SignalError> {
    verify_locality_case(definition, 0, DiagnosticsTier::Operational, 1)
}

fn certify_index_disjoint(
    lower: FinancialLocalityCaseEvidence,
    upper: FinancialLocalityCaseEvidence,
) -> Result<InvalidationCostSlopeReport, SignalError> {
    require_scenario(&lower, FinancialLocalityScenario::SparseBookFanout)?;
    require_scenario(&upper, FinancialLocalityScenario::SparseBookFanout)?;
    if lower.counters() != upper.counters()
        || lower.necessary_evaluations() != upper.necessary_evaluations()
    {
        return Err(SignalError::internal(
            "index-disjoint growth changed performed hot work",
        ));
    }
    Ok(report(
        InvalidationCostSlopeClass::IndexDisjoint,
        lower,
        upper,
        0,
        0,
    ))
}

fn certify_queried_candidate(
    lower: FinancialLocalityCaseEvidence,
    upper: FinancialLocalityCaseEvidence,
    expected_delta: u64,
) -> Result<InvalidationCostSlopeReport, SignalError> {
    let lower_rows = lower.counters();
    let upper_rows = upper.counters();
    for row in [
        Counter::DirectSubscriberEdgesExamined,
        Counter::ReverseIndexCandidatesReturned,
        Counter::CandidatesRejectedByScope,
    ] {
        require_delta(
            row.name(),
            lower_rows.value(row),
            upper_rows.value(row),
            expected_delta,
        )?;
    }
    for row in [
        Counter::DirectSettlementsProduced,
        Counter::WorkItemsAdmitted,
        Counter::ReadyItemsEnqueued,
        Counter::ReadyItemsPopped,
        Counter::NodesEvaluated,
    ] {
        require_delta(row.name(), lower_rows.value(row), upper_rows.value(row), 0)?;
    }
    if lower.necessary_evaluations() != upper.necessary_evaluations() {
        return Err(SignalError::internal(
            "queried-candidate growth changed semantic work identity",
        ));
    }
    Ok(report(
        InvalidationCostSlopeClass::QueriedCandidate,
        lower,
        upper,
        expected_delta,
        0,
    ))
}

fn certify_semantic_frontier(
    lower: FinancialLocalityCaseEvidence,
    upper: FinancialLocalityCaseEvidence,
    expected_delta: u64,
) -> Result<InvalidationCostSlopeReport, SignalError> {
    let lower_rows = lower.counters();
    let upper_rows = upper.counters();
    for row in [
        Counter::DirectSubscriberEdgesExamined,
        Counter::ReverseIndexCandidatesReturned,
        Counter::DirectSettlementsProduced,
        Counter::WorkItemsAdmitted,
        Counter::ReadyItemsEnqueued,
        Counter::ReadyItemsPopped,
        Counter::NodesEvaluated,
    ] {
        require_delta(
            row.name(),
            lower_rows.value(row),
            upper_rows.value(row),
            expected_delta,
        )?;
    }
    require_delta(
        "necessary_evaluations",
        lower.necessary_evaluations().len() as u64,
        upper.necessary_evaluations().len() as u64,
        expected_delta,
    )?;
    Ok(report(
        InvalidationCostSlopeClass::SemanticFrontier,
        lower,
        upper,
        expected_delta,
        expected_delta,
    ))
}

fn report(
    class: InvalidationCostSlopeClass,
    lower: FinancialLocalityCaseEvidence,
    upper: FinancialLocalityCaseEvidence,
    added_candidates: u64,
    added_semantic_work: u64,
) -> InvalidationCostSlopeReport {
    InvalidationCostSlopeReport {
        class,
        lower_case: lower.into_identity(),
        upper_case: upper.into_identity(),
        added_candidates,
        added_semantic_work,
    }
}

fn require_scenario(
    evidence: &FinancialLocalityCaseEvidence,
    scenario: FinancialLocalityScenario,
) -> Result<(), SignalError> {
    if evidence.scenario() == scenario {
        Ok(())
    } else {
        Err(SignalError::internal("cost slope mixed scenario families"))
    }
}

fn require_delta(row: &str, lower: u64, upper: u64, expected: u64) -> Result<(), SignalError> {
    if upper.checked_sub(lower) == Some(expected) {
        Ok(())
    } else {
        Err(SignalError::internal(format!(
            "cost slope {row} delta drifted: lower={lower}, upper={upper}, expected={expected}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_reports_keep_the_three_locality_slopes_separate() {
        let reports = certify_ordinary_cost_slopes(41).unwrap();
        assert_eq!(
            reports[0].class(),
            InvalidationCostSlopeClass::IndexDisjoint
        );
        assert_eq!(reports[0].added_candidates(), 0);
        assert_eq!(
            reports[1].class(),
            InvalidationCostSlopeClass::QueriedCandidate
        );
        assert_eq!(reports[1].added_candidates(), 448);
        assert_eq!(reports[1].added_semantic_work(), 0);
        assert_eq!(
            reports[2].class(),
            InvalidationCostSlopeClass::SemanticFrontier
        );
        assert_eq!(reports[2].added_semantic_work(), 7);
        assert!(reports
            .iter()
            .all(|report| report.case_identities().0 != report.case_identities().1));
    }
}
