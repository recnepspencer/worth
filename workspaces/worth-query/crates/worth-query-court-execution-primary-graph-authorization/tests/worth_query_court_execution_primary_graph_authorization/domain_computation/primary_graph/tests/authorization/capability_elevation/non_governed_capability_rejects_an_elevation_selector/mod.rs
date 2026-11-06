//! non governed capability rejects an elevation selector.
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn non_governed_capability_rejects_an_elevation_selector() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    let projection = ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            AccountIdentity::at("AccountIdentity"),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "account-1".to_owned(),
            )
            .expect("fixture account identity must encode"),
        ).unwrap(),
        ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(CapabilityAction::Touch)
            .expect("fixture capability action must encode"),
        ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
            CapabilityPurpose::AccountMaintenance,
        )
        .expect("fixture capability purpose must encode"),
        ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()),
    )
    .elevation(ApplicationCapabilityEntitySelector::new(
        CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
        ApplicationEncodedScalarValue::<<CapabilityElevationIdentity as worth_query_declaration::facade::application_schema::fragment::RecordSelection>::Binding>::try_new("elevation-1".to_owned().into())
        .expect("fixture elevation identity must encode"),
    ).unwrap());

    let denial = worth_query_execution::court_doors_for_test::domain_computation::authorization::capability_elevation_projection::validate_elevation_projection(
        capability.contract(),
        &projection,
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationNotApplicable
        )
    );
}
