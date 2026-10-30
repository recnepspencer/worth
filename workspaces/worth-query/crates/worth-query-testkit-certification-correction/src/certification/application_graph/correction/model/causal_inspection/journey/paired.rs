//! Original disclosed values determine comparison, while the request owns funding.
use super::*;
use worth_query_host::facade::runtime::{
    CancellationToken, DeterminismContract, ExecutionAuthority, ExecutionPosture,
    ExecutionRequestPolicy, LeaseRequest,
};

pub fn changed_and_rewritten_equal_originals_keep_the_request_payer() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let readers = application.application_query_basis_observer();
    let baseline_readers = readers.observe().active();
    let (before, row) = read(&application, &principal, &scope, None);
    assert_eq!((row.input, row.calculated), (5, 0));
    let (_, _, basis) = perform(&application, 11, "paired-first-eleven");
    let (after, row) = read(&application, &principal, &scope, Some(&basis));
    assert_eq!((row.input, row.calculated), (11, 23));
    drop(basis);
    let _ = perform(&application, 7, "paired-intervening-seven");
    let (_, _, basis) = perform(&application, 11, "paired-rewritten-eleven");
    let (equal, row) = read(&application, &principal, &scope, Some(&basis));
    assert_eq!((row.input, row.calculated), (11, 23));
    drop(basis);
    let authority = ExecutionAuthority::serial();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(32 * 1024 * 1024),
                    Some(100_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let baseline = lease.charged_memory_bytes();
    let request = application.runtime().request(&principal, &scope);
    for (left, right, reason) in [
        (&before, &after, CausalInspectionReason::ChangedResult),
        (&after, &equal, CausalInspectionReason::UnchangedResult),
    ] {
        let report = request
            .inspect::<query::Binding>(right)
            .why_changed(left)
            .reference_only()
            .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
            .execute_governed(
                access(&application, &principal, &scope),
                access(&application, &principal, &scope),
            )
            .unwrap();
        assert_eq!(report.reason(), reason);
        assert!(std::ptr::eq(report.before(), left));
        assert!(std::ptr::eq(report.after(), right));
        assert_ne!(
            report.before_report_identity(),
            report.after_report_identity()
        );
        assert_ne!(report.report_identity(), report.before_report_identity());
        assert!(lease.charged_memory_bytes() > baseline);
        assert_eq!(readers.observe().active(), baseline_readers);
        report.close();
        assert_eq!(lease.charged_memory_bytes(), baseline);
    }
    before.close();
    after.close();
    equal.close();
    assert_eq!(readers.observe().active(), baseline_readers);
}
