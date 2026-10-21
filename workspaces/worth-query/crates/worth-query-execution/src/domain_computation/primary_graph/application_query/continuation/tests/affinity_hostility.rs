use std::time::Duration;

use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind;

use super::support::ContinuationTestContext;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind;

#[test]
fn changed_parameter_denies_before_basis_acquisition() {
    let context = ContinuationTestContext::new(Duration::from_secs(60));
    let continuation = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let acquisitions = context.basis_acquisitions();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();

    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.readmit_denial(&host_custody, continuation, "account-2", &request, 1)
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial,
        WorthQueryApplicationQueryAdmissionDenialKind::ContinuationParameterMismatch
    );
    assert_eq!(context.basis_acquisitions(), acquisitions);
    context.assert_resource_baseline();
}

#[test]
fn stale_installed_generation_denies_before_basis_reacquisition() {
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
    context.advance_installation();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();

    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.readmit_denial(&host_custody, continuation, "account-1", &request, 1)
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial,
        WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
            WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
        )
    );
    assert_eq!(context.basis_acquisitions(), acquisitions);
    context.assert_resource_baseline();
}

#[test]
fn wrong_provider_descriptor_denies_before_fresh_admission() {
    let context = ContinuationTestContext::new(Duration::from_secs(60));
    let mut continuation = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    continuation.provider_identity.push_str(".foreign");
    let acquisitions = context.basis_acquisitions();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();

    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.readmit_denial(&host_custody, continuation, "account-1", &request, 1)
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial,
        WorthQueryApplicationQueryAdmissionDenialKind::ContinuationProviderMismatch
    );
    assert_eq!(context.basis_acquisitions(), acquisitions);
    context.assert_resource_baseline();
}

#[test]
fn changed_continuation_contract_denies_as_stale_meaning() {
    let context = ContinuationTestContext::new(Duration::from_secs(60));
    let mut continuation = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    continuation.continuation_contract_digest =
        worth_foundational::facade::CanonicalDigestId::new([0xCC; 32]);
    let acquisitions = context.basis_acquisitions();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();

    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.readmit_denial(&host_custody, continuation, "account-1", &request, 1)
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial,
        WorthQueryApplicationQueryAdmissionDenialKind::StaleContinuation
    );
    assert_eq!(context.basis_acquisitions(), acquisitions);
    context.assert_resource_baseline();
}

#[test]
fn retained_and_disposed_tokens_own_no_runtime_resource_types() {
    let context = ContinuationTestContext::new(Duration::from_secs(60));
    let continuation = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.issue(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    context.assert_resource_baseline();
    drop(continuation);
    context.assert_resource_baseline();

    let token_source = include_str!("../authority.rs");
    for forbidden in [
        "RelationalExecutionBasisLease",
        "WorthQueryApplicationBasisLease",
        "WorthQueryProviderSession",
        "WorthQueryApplicationResultBuffer",
        "WorthQueryManagedLowerExecutionBasis",
        "WorthQueryLiveCauseQueue",
    ] {
        assert!(
            !token_source.contains(forbidden),
            "continuation token retained forbidden resource type {forbidden}"
        );
    }
}

#[test]
fn continuation_parameter_affinity_retains_and_compares_foundational_basis() {
    let token_source = include_str!("../authority.rs");
    let readmission_source = include_str!("../readmission.rs");

    assert!(token_source.contains("WorthQueryApplicationParameterCanonicalArtifact"));
    assert!(!token_source.contains("parameter_identity: String"));
    assert!(readmission_source.contains(".is_equivalent_to(&affinity.parameter_basis)"));
    assert!(!readmission_source.contains("parameters.identity() !="));
}
