//! Existing execution leases independently observe the source report's lifetime.
use super::*;
pub fn source_report_retains_fresh_request_funding_until_close() {
    use std::num::NonZeroUsize;
    use worth_query_host::facade::runtime::{
        CancellationToken, DeterminismContract, ExecutionAllocationPolicy, ExecutionAuthority,
        ExecutionPosture, ExecutionRequestPolicy, LeaseRequest,
    };
    let application = host::install();
    let source = source(&application, "source-only-payer");
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let authority = ExecutionAuthority::serial();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    Some(32 * 1024 * 1024),
                    Some(100_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let before = lease.charged_memory_bytes();
    let report = request
        .inspect_source::<SourceOnlyDisclosure>(&source)
        .why_performed()
        .reference_only()
        .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
        .execute_governed(journey::access(&application, &principal, &scope))
        .unwrap();
    assert!(
        lease.charged_memory_bytes() > before,
        "the returned source artifact retains its actual request reservation"
    );
    assert_eq!(readers.observe().active(), baseline);
    drop(source);
    assert!(lease.charged_memory_bytes() > before);
    report.close();
    assert_eq!(lease.charged_memory_bytes(), before);
    assert_eq!(readers.observe().active(), baseline);
}
