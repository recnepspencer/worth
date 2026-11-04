//! Recovery authority issuance preserves the pre-cutover interrupted-request behavior.
use crate::domain_computation::primary_graph::tests::application_attempt::recovery_custody::*;

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
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        ))
    );
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .expect("interruption cannot add a new recovery issuance refusal");
    worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::recovery_progression::authority::require_fresh_effect_authority(
        &handle, &authority,
    )
    .expect("the issued authority belongs to this genuinely live handle");
}
