use crate::data::telemetry::InvalidationPerformedCounter;
use crate::facade::DiagnosticsTier;
use crate::tests::domains::fintech::{verify_locality_case, FinancialWorldDefinition};

#[test]
fn disjoint_region_growth_does_not_expand_semantic_work() {
    let lower = verify_locality_case(
        FinancialWorldDefinition::partitioned_curve_universe(41, 4, 1, 1),
        0,
        DiagnosticsTier::Operational,
        1,
    )
    .expect("lower region scale should settle");
    let upper = verify_locality_case(
        FinancialWorldDefinition::partitioned_curve_universe(41, 16, 1, 1),
        0,
        DiagnosticsTier::Operational,
        1,
    )
    .expect("upper region scale should settle");

    assert_eq!(
        lower.necessary_evaluation_count(),
        upper.necessary_evaluation_count()
    );
    assert_eq!(lower.canonical_work_items(), upper.canonical_work_items());
    assert_eq!(
        lower
            .counters()
            .value(InvalidationPerformedCounter::NodesEvaluated),
        upper
            .counters()
            .value(InvalidationPerformedCounter::NodesEvaluated)
    );
    assert_eq!(
        lower
            .counters()
            .value(InvalidationPerformedCounter::WorkItemsAdmitted),
        upper
            .counters()
            .value(InvalidationPerformedCounter::WorkItemsAdmitted)
    );
}
