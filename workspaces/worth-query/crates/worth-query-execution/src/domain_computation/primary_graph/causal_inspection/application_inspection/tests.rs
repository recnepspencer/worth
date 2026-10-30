//! Genuine completed queries enter the same fresh ordinary/governed issuer.
use super::super::{
    CausalInspectionReason, CausalObservationMissingReferencePosture,
    WorthQueryApplicationQueryObservation,
};
use super::*;
use crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernanceDenialKind;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        admit_touch_account_capability, installed_authorization_world,
        installed_inspection_capability_world, live_scope, status_parameter, AccountStatus,
        AlternateAccountSourceBinding, AuthorizationWorld, IdentityExecutionSchema,
        TestAccountSourceBinding,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use std::{
    num::NonZeroUsize,
    time::{Duration, UNIX_EPOCH},
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

pub(super) fn observe(
    world: &AuthorizationWorld,
    external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
    request: &WorthQueryRequestScope,
    governed: bool,
) -> WorthQueryApplicationQueryObservation {
    let query = if governed {
        world
            .application
            .installed_schema()
            .installed_query_binding::<AlternateAccountSourceBinding>()
            .unwrap()
            .into_query()
    } else {
        world
            .application
            .installed_schema()
            .installed_query_binding::<TestAccountSourceBinding>()
            .unwrap()
            .into_query()
    };
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    external,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let scope = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let plan = selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .unwrap(),
                    WorthQueryProductQueryControls::new(NonZeroUsize::MIN, request),
                )
                .unwrap();
            let completed = world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source();
            assert_eq!(completed.receipt().result_count(), 1);
            completed.retain_inspection_observation().unwrap()
        })
        .unwrap()
}

#[test]
fn fresh_observed_reference_uses_original_request_funding_and_releases_every_reader() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let original = observe(&world, &external, &request, false);
    let registry = world.application.application_query_basis_observer();
    let readers = registry.observe().active();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            Some(32 * 1024 * 1024),
            Some(100_000_000),
        ),
    );
    let authority = worth_execution::ExecutionAuthority::serial();
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy,
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let before = lease.charged_memory_bytes();
    let explanation = world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(&external, &request, &original)
        .why_observed()
        .reference_only()
        .with_allocation_policy(worth_execution::ExecutionAllocationPolicy::Execution(
            &lease,
        ))
        .execute()
        .unwrap();
    assert!(std::ptr::eq(explanation.observation(), &original));
    assert_eq!(explanation.reason(), CausalInspectionReason::ObservedResult);
    assert_eq!(
        explanation.lower_evidence_posture(),
        CausalObservationMissingReferencePosture::MissingOptional
    );
    assert!(!explanation.observation_identity().is_empty());
    assert!(!explanation.report_identity().is_empty());
    assert!(
        lease.charged_memory_bytes() > before,
        "live report retains its original ancestor ticket"
    );
    assert_eq!(
        registry.observe().active(),
        readers,
        "descriptive output retains no reader"
    );
    explanation.close();
    assert_eq!(lease.charged_memory_bytes(), before);
    assert_eq!(registry.observe().active(), readers);
}

#[test]
fn fresh_inspection_refuses_wrong_binding_foreign_runtime_and_original_cancellation() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let original = observe(&world, &external, &request, false);
    let wrong = world
        .application
        .inspect_application_observation::<AlternateAccountSourceBinding>(
            &external, &request, &original,
        )
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .unwrap();
    assert!(wrong.producer_output_refusal().is_none());
    assert!(matches!(
        wrong.cause,
        stop::InspectionCause::Admission(admission::InspectionAdmissionDenial::ForeignIssuedAccess)
    ));
    let foreign = installed_authorization_world(true);
    let denied = foreign
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(&external, &request, &original)
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .unwrap();
    assert!(denied.producer_output_refusal().is_none());
    assert!(matches!(
        denied.cause,
        stop::InspectionCause::Admission(admission::InspectionAdmissionDenial::ForeignIssuedAccess)
    ));
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let cause = world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(&external, &stopped, &original)
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .unwrap();
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    assert!(cause.producer_output_refusal().is_none());
    let lawful = world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(&external, &request, &original)
        .why_observed()
        .reference_only()
        .execute()
        .unwrap();
    lawful.close();
}

#[test]
fn governed_inspection_requires_fresh_actual_capability_independent_of_public_query() {
    let world = installed_inspection_capability_world();
    world
        .authorization_time
        .hold(UNIX_EPOCH + Duration::from_secs(100));
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let original = observe(&world, &external, &request, true);
    let missing = world
        .application
        .inspect_application_observation::<AlternateAccountSourceBinding>(
            &external, &request, &original,
        )
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .unwrap();
    assert!(missing.producer_output_refusal().is_none());
    assert!(matches!(
        missing.cause,
        stop::InspectionCause::Admission(admission::InspectionAdmissionDenial::Governance(
            WorthQueryApplicationQueryGovernanceDenialKind::Required
        ))
    ));
    let capability = world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            admit_touch_account_capability(&phase.test_host_request(), &world, &principal, &request)
                .unwrap()
        })
        .unwrap();
    let explanation = world
        .application
        .inspect_application_observation::<AlternateAccountSourceBinding>(
            &external, &request, &original,
        )
        .why_observed()
        .reference_only()
        .execute_governed(capability)
        .unwrap();
    assert_eq!(explanation.reason(), CausalInspectionReason::ObservedResult);
    explanation.close();
    let public_original = observe(&world, &external, &request, false);
    let retained_capability = world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            admit_touch_account_capability(&phase.test_host_request(), &world, &principal, &request)
                .unwrap()
        })
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            crate::domain_computation::primary_graph::tests::fixture::revoke_current_capability(
                &host,
                &host.borrowed_request_for_test(),
                &world,
            );
        })
        .unwrap();
    let revoked = world
        .application
        .inspect_application_observation::<AlternateAccountSourceBinding>(
            &external, &request, &original,
        )
        .why_observed()
        .reference_only()
        .execute_governed(retained_capability)
        .err()
        .unwrap();
    assert!(revoked.producer_output_refusal().is_none());
    let stop::InspectionCause::Admission(admission::InspectionAdmissionDenial::Capability(cause)) =
        &revoked.cause
    else {
        panic!("actual retained capability must be freshly revalidated")
    };
    assert_eq!(cause.kind(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleAuthorization));
    let lawful = world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(
            &external,
            &request,
            &public_original,
        )
        .why_observed()
        .reference_only()
        .execute()
        .unwrap();
    lawful.close();
}
