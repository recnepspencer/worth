//! Real original query funding survives one holder, and ends at the final holder.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryCancellationSource, runtime::*,
};

pub fn availability_keeps_sibling_custody_and_releases_final_funding() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let (source, producer, read_basis) = perform(&application, 11, "availability-funded-eleven");
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
    let original_charge = lease.charged_memory_bytes();
    let (original, row) = retained_query::read_funded(
        &application,
        &principal,
        &scope,
        &read_basis,
        ExecutionAllocationPolicy::Execution(&lease),
    );
    assert_eq!((row.input, row.calculated), (11, 23));
    drop((source, producer, read_basis, row));
    assert_eq!(readers.observe().active(), baseline);
    let retained_charge = lease.charged_memory_bytes();
    assert!(retained_charge > original_charge);
    let sibling = original.clone();
    original.close();
    assert_eq!(
        lease.charged_memory_bytes(),
        retained_charge,
        "the first close must not release a sibling's original reservation"
    );
    let report = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&sibling)
        .why_observed()
        .reference_only()
        .execute_governed(access(&application, &principal, &scope))
        .unwrap();
    let availability = report.evidence_availability();
    assert_eq!(
        availability.projection(),
        WorthQueryInspectionRetainedEvidence::Retained
    );
    assert_eq!(
        availability.supplier(),
        WorthQueryInspectionSupplierEvidence::CommittedOutput
    );
    assert_eq!(
        availability.native_observation(),
        WorthQueryInspectionRetainedEvidence::Retained
    );
    report.close();
    assert_eq!(lease.charged_memory_bytes(), retained_charge);

    // Actual request cancellation cannot expire the independently retained original.
    let admitted = access(&application, &principal, &scope);
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    let denied = application
        .runtime()
        .request(&principal, &cancelled)
        .inspect::<query::Binding>(&sibling)
        .why_observed()
        .reference_only()
        .execute_governed(admitted)
        .err()
        .expect("cancelled metadata request refuses");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    drop(denied);
    assert_eq!(lease.charged_memory_bytes(), retained_charge);
    let survivor = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&sibling)
        .why_observed()
        .reference_only()
        .execute_governed(access(&application, &principal, &scope))
        .unwrap();
    assert_eq!(survivor.evidence_availability(), availability);
    survivor.close();

    // A real grant withdrawal precedes protected availability lookup.
    let admitted = access(&application, &principal, &scope);
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(super::super::distinct_subjects::grant_status::Withdraw {
            grant: "inspection-grant".to_owned(),
        })
        .without_source()
        .idempotency(&"availability-withdraw-grant".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let denied = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&sibling)
        .why_observed()
        .reference_only()
        .execute_governed(admitted)
        .err()
        .expect("withdrawn metadata cannot reveal availability");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denied.producer_connection_refusal().is_none());
    drop(denied);
    assert_eq!(lease.charged_memory_bytes(), retained_charge);
    sibling.close();
    assert_eq!(
        lease.charged_memory_bytes(),
        original_charge,
        "no global owner may keep the original query funding after its final close"
    );
    assert_eq!(readers.observe().active(), baseline);
}
