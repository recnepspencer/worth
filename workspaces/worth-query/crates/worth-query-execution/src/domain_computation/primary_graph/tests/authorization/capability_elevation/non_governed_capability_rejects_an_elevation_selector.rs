//! non governed capability rejects an elevation selector.
use super::*;

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
            AccountIdentity::reference(),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "account-1".to_owned(),
            )
            .expect("fixture account identity must encode"),
        ),
        ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(CapabilityAction::Touch)
            .expect("fixture capability action must encode"),
        ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
            CapabilityPurpose::AccountMaintenance,
        )
        .expect("fixture capability purpose must encode"),
        ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()),
    )
    .elevation(ApplicationCapabilityEntitySelector::new(
        CapabilityElevationIdentity::reference(),
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
            "elevation-1".to_owned(),
        )
        .expect("fixture elevation identity must encode"),
    ));

    let denial = crate::domain_computation::authorization::validate_elevation_projection(
        capability.contract(),
        &projection,
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationNotApplicable
    );
}
