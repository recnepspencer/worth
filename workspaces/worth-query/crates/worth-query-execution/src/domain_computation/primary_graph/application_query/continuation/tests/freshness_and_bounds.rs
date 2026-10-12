use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::support::ContinuationTestContext;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind;

#[test]
fn authentication_expiry_denies_resume_without_a_basis() {
    let mut context = ContinuationTestContext::new(Duration::from_secs(60));
    let continuation = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let acquisitions = context.basis_acquisitions();
    context.expire_authentication();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();

    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.readmit_denial(
                &host_custody,
                continuation,
                "account-1",
                &request,
                1,
                10_000,
            )
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial,
        WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal
    );
    assert_eq!(context.basis_acquisitions(), acquisitions);
    context.assert_resource_baseline();
}

#[test]
fn stale_principal_and_scope_deny_and_release_readmission_bases() {
    let principal_context = ContinuationTestContext::new(Duration::from_secs(60));
    let principal_continuation = principal_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            principal_context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let principal_acquisitions = principal_context.basis_acquisitions();
    principal_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            principal_context.stale_principal_mapping(&host_custody.borrowed_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let after_principal_publication = principal_context.basis_acquisitions();
    assert_eq!(after_principal_publication, principal_acquisitions + 1);
    let principal_request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let principal = principal_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            principal_context.readmit_denial(
                &host_custody,
                principal_continuation,
                "account-1",
                &principal_request,
                1,
                10_000,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        principal,
        WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal
    );
    assert_eq!(
        principal_context.basis_acquisitions(),
        after_principal_publication + 2,
        "readmission acquires one exact data lease and one fresh security lease, then releases both",
    );
    principal_context.assert_resource_baseline();

    let scope_context = ContinuationTestContext::new(Duration::from_secs(60));
    let scope_continuation = scope_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            scope_context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let scope_acquisitions = scope_context.basis_acquisitions();
    scope_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            scope_context.stale_scope_identity(&host_custody.borrowed_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let after_scope_publication = scope_context.basis_acquisitions();
    assert_eq!(after_scope_publication, scope_acquisitions + 1);
    let scope_request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let scope = scope_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            scope_context.readmit_denial(
                &host_custody,
                scope_continuation,
                "account-1",
                &scope_request,
                1,
                10_000,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        scope,
        WorthQueryApplicationQueryAdmissionDenialKind::StaleScope
    );
    assert_eq!(
        scope_context.basis_acquisitions(),
        after_scope_publication + 2,
        "readmission acquires one exact data lease and one fresh security lease, then releases both",
    );
    scope_context.assert_resource_baseline();
}

#[test]
fn cancellation_and_deadline_deny_resume_before_basis_acquisition() {
    let cancelled_context = ContinuationTestContext::new(Duration::from_secs(60));
    let cancelled_continuation = cancelled_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            cancelled_context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let cancelled_acquisitions = cancelled_context.basis_acquisitions();
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled_request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let cancelled = cancelled_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            cancelled_context.readmit_denial(
                &host_custody,
                cancelled_continuation,
                "account-1",
                &cancelled_request,
                1,
                10_000,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        cancelled,
        WorthQueryApplicationQueryAdmissionDenialKind::Cancelled
    );
    assert_eq!(
        cancelled_context.basis_acquisitions(),
        cancelled_acquisitions
    );
    cancelled_context.assert_resource_baseline();

    let deadline_context = ContinuationTestContext::new(Duration::from_secs(60));
    let deadline_continuation = deadline_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            deadline_context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let deadline_acquisitions = deadline_context.basis_acquisitions();
    let deadline_source = WorthQueryCancellationSource::new();
    let deadline_request =
        WorthQueryRequestScope::new(Some(Instant::now()), deadline_source.token());
    let deadline = deadline_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            deadline_context.readmit_denial(
                &host_custody,
                deadline_continuation,
                "account-1",
                &deadline_request,
                1,
                10_000,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        deadline,
        WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded
    );
    assert_eq!(deadline_context.basis_acquisitions(), deadline_acquisitions);
    deadline_context.assert_resource_baseline();
}

#[test]
fn foreign_continuation_basis_denies_without_registering_a_lease() {
    let context = ContinuationTestContext::new(Duration::from_secs(60));
    let mut continuation = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let foreign = ContinuationTestContext::new(Duration::from_secs(60));
    foreign
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            continuation.product = foreign.issue(&host_custody).product
        })
        .expect("the fixture read owns its installed host call");
    let acquisitions = context.basis_acquisitions();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();

    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.readmit_denial(
                &host_custody,
                continuation,
                "account-1",
                &request,
                1,
                10_000,
            )
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial,
        WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
    );
    assert_eq!(context.basis_acquisitions(), acquisitions);
    context.assert_resource_baseline();
}

#[test]
fn page_and_work_exhaustion_mint_no_resumed_plan_or_basis() {
    let width_context = ContinuationTestContext::new(Duration::from_secs(60));
    let width_continuation = width_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            width_context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let width_acquisitions = width_context.basis_acquisitions();
    let width_request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let width = width_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            width_context.readmit_denial(
                &host_custody,
                width_continuation,
                "account-1",
                &width_request,
                257,
                10_000,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        width,
        WorthQueryApplicationQueryAdmissionDenialKind::ContinuationPageWidthUnsupported
    );
    assert_eq!(width_context.basis_acquisitions(), width_acquisitions);
    width_context.assert_resource_baseline();

    let work_context = ContinuationTestContext::new(Duration::from_secs(60));
    let work_continuation = work_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            work_context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let work_acquisitions = work_context.basis_acquisitions();
    let work_request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let work = work_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            work_context.readmit_denial(
                &host_custody,
                work_continuation,
                "account-1",
                &work_request,
                1,
                1,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        work,
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded
    );
    assert_eq!(work_context.basis_acquisitions(), work_acquisitions);
    work_context.assert_resource_baseline();
}
