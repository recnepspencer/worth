use crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationDisclosureReceipt;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity;

#[test]
fn application_outcome_families_draw_non_aliasing_exact_identities() {
    let denial = WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied),
        "identity-family-denial",
    );
    let disclosure = WorthQueryApplicationDisclosureReceipt::governed(
        "identity-family-disclosure",
        Vec::new(),
        Vec::new(),
        "installed-capability-authority",
        [7; 32],
        0,
    );
    let commit = WorthQueryApplicationCommitOutcomeIdentity::mint().unwrap();

    let denial = denial.identity().unwrap().get();
    let disclosure = disclosure.outcome_identity().unwrap().get();
    assert_ne!(denial, disclosure);
    assert_ne!(denial, commit.get());
    assert_ne!(disclosure, commit.get());
}
