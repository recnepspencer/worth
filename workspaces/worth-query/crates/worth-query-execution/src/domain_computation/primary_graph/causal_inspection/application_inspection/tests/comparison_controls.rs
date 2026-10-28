//! Current policy gates precede original availability, including zero rows.
use super::*;

#[test]
fn zero_rows_do_not_bypass_current_internal_relation_or_field_explanation_policy() {
    let world = comparison_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let registry = world.application.application_query_basis_observer();
    let baseline = registry.observe().active();
    fn denied<B: DiagnosticBinding>(
        world: &AuthorizationWorld,
        external: &WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema>,
        request: &WorthQueryRequestScope,
        expected: DiagnosticComparisonDenial,
    ) {
        let (before, rows) = read::<B>(world, external, request, "no-match", false);
        assert!(rows.is_empty());
        let (after, rows) = read::<B>(world, external, request, "no-match", false);
        assert!(rows.is_empty());
        let denied = compare::<B>(world, external, request, &before, &after)
            .err()
            .expect("the real comparison gate must refuse");
        let stop::InspectionCause::Admission(
            admission::InspectionAdmissionDenial::DiagnosticAdmission(CaptureStop::Kernel(
                ScopeStop::Failure(MapKernelFailure::Domain(actual)),
            )),
        ) = &denied.cause
        else {
            panic!("current cold policy must refuse before missing original bodies: {denied:?}");
        };
        assert_eq!(actual, &expected);
    }
    denied::<DeniedDiagnosticInternalBinding>(
        &world,
        &external,
        &request,
        DiagnosticComparisonDenial::InternalInfluenceDenied,
    );
    denied::<DeniedDiagnosticRelationBinding>(
        &world,
        &external,
        &request,
        DiagnosticComparisonDenial::RelationDenied,
    );
    denied::<DeniedDiagnosticFieldBinding>(
        &world,
        &external,
        &request,
        DiagnosticComparisonDenial::FieldDenied,
    );
    let (before, rows) =
        read::<DiagnosticAccountBinding>(&world, &external, &request, "no-match", true);
    assert!(rows.is_empty());
    let (after, rows) =
        read::<DiagnosticAccountBinding>(&world, &external, &request, "no-match", true);
    assert!(rows.is_empty());
    let result =
        compare::<DiagnosticAccountBinding>(&world, &external, &request, &before, &after).unwrap();
    assert_eq!(result.reason(), CausalInspectionReason::UnchangedResult);
    result.close();
    // Public metadata cannot stand in for the independently current diagnostic capability.
    let (bodyless_before, _) =
        read::<DiagnosticAccountBinding>(&world, &external, &request, "no-match", false);
    let (bodyless_after, _) =
        read::<DiagnosticAccountBinding>(&world, &external, &request, "no-match", false);
    let retained_diagnostic = capability(&world, &external, &request);
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
        .inspect_application_observation::<DiagnosticAccountBinding>(
            &external,
            &request,
            &bodyless_after,
        )
        .why_changed(&bodyless_before)
        .reference_only()
        .execute_with_diagnostic(retained_diagnostic)
        .err()
        .expect("revoked diagnostic capability must refuse before missing-body lookup");
    let stop::InspectionCause::Admission(admission::InspectionAdmissionDenial::Capability(cause)) =
        &revoked.cause
    else {
        panic!("the actual diagnostic authorization must be revalidated: {revoked:?}");
    };
    assert_eq!(cause.kind(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleAuthorization));
    drop(revoked);
    assert_eq!(registry.observe().active(), baseline);
}

#[test]
fn paired_unavailable_and_foreign_parameter_inputs_keep_their_real_gate_order() {
    let world = comparison_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let (before, _) = read::<DiagnosticAccountBinding>(&world, &external, &request, "open", false);
    let (after, _) = read::<DiagnosticAccountBinding>(&world, &external, &request, "open", false);
    let missing = compare::<DiagnosticAccountBinding>(&world, &external, &request, &before, &after)
        .err()
        .expect("the real comparison gate must refuse");
    assert_eq!(
        missing.kind(),
        WorthQueryApplicationInspectionStopKind::Unavailable
    );
    assert!(matches!(
        &missing.cause,
        stop::InspectionCause::Admission(
            admission::InspectionAdmissionDenial::MissingOriginalProjection
        )
    ));
    let (different_parameters, rows) =
        read::<DiagnosticAccountBinding>(&world, &external, &request, "no-match", false);
    assert!(rows.is_empty());
    let different = compare::<DiagnosticAccountBinding>(
        &world,
        &external,
        &request,
        &before,
        &different_parameters,
    )
    .err()
    .expect("the real comparison gate must refuse");
    let stop::InspectionCause::Admission(
        admission::InspectionAdmissionDenial::ComparisonParameters(stop),
    ) = &different.cause
    else {
        panic!("cross-original parameters must refuse after current diagnostic policy");
    };
    assert!(matches!(
        stop,
        CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(())))
    ));
    let (foreign, _) =
        read::<GovernedDiagnosticAccountBinding>(&world, &external, &request, "open", false);
    let foreign =
        compare::<DiagnosticAccountBinding>(&world, &external, &request, &foreign, &after)
            .err()
            .expect("the real comparison gate must refuse");
    assert!(matches!(
        &foreign.cause,
        stop::InspectionCause::Admission(admission::InspectionAdmissionDenial::ForeignIssuedAccess)
    ));
    let public = super::tests::observe(&world, &external, &request, false);
    let diagnostic = capability(&world, &external, &request);
    let public = world
        .application
        .inspect_application_observation::<TestAccountSourceBinding>(&external, &request, &public)
        .why_changed(&public)
        .reference_only()
        .execute_with_diagnostic(diagnostic)
        .err()
        .expect("the real comparison gate must refuse");
    assert_eq!(
        public.kind(),
        WorthQueryApplicationInspectionStopKind::Unavailable
    );
    let stop::InspectionCause::Admission(admission::InspectionAdmissionDenial::DiagnosticContract(
        stop,
    )) = &public.cause
    else {
        panic!("Public payload must refuse diagnostic comparison before old-body lookup");
    };
    assert!(matches!(stop, CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(
        crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::NotDeclared)))));
}
