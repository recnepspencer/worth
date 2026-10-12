use worth_foundational::FoundationalPerformanceWorkClass;

use super::execution::settled_honest_execution;

#[test]
fn settled_cost_snapshot_exports_exact_distinct_domain_counter_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let settled = settled_honest_execution(request_execution, "domain-evidence-cost");
    let snapshot = settled.consumption_cost_snapshot();

    for (name, expected) in [
        ("domain.operation.bytes", 128),
        ("domain.operation.elements", 4),
        ("domain.operation.candidate-comparisons", 6),
        ("domain.operation.operation-work-components", 10),
        ("domain.operation.work", 10),
    ] {
        let row = snapshot
            .row(name)
            .unwrap_or_else(|| panic!("missing admitted domain counter row {name}"));
        assert_eq!(row.observed_count(), expected, "wrong count for {name}");
        assert_eq!(
            row.work_class(),
            FoundationalPerformanceWorkClass::AuthoritativeObservation,
            "wrong work class for {name}"
        );
    }

    let domain_names = snapshot
        .rows()
        .iter()
        .filter(|row| row.name().starts_with("domain.operation."))
        .map(|row| row.name())
        .collect::<Vec<_>>();
    assert_eq!(domain_names.len(), 5);
    let mut unique = domain_names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), domain_names.len());
}
