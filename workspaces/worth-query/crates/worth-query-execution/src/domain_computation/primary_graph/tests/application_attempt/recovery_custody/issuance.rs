//! Recovery authority issuance preserves the pre-cutover interrupted-request behavior.
use super::*;

#[test]
fn cancelled_admission_can_issue_recovery_authority_for_its_live_handle() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (handle, admission, cancellation) = committed_handle(&world);
    cancellation.cancel();
    assert_eq!(
        admission.validate_current_authority().unwrap_err().kind(),
        WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        ))
    );
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .expect("interruption cannot add a new recovery issuance refusal");
    crate::domain_computation::application_aftermath::require_fresh_effect_authority(
        &handle, &authority,
    )
    .expect("the issued authority belongs to this genuinely live handle");
}
