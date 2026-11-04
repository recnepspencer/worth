use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcomeIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosureReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

#[test]
fn application_outcome_families_draw_non_aliasing_exact_identities() {
    let denial = WorthQueryOperationAuthorizationDenial::new_for_test(
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied),
        "identity-family-denial",
    );
    let disclosure = WorthQueryApplicationDisclosureReceipt::governed_for_test(
        "identity-family-disclosure",
        Vec::new(),
        Vec::new(),
        "installed-capability-authority",
        [7; 32],
        0,
    );
    let commit = WorthQueryApplicationCommitOutcomeIdentity::mint_for_test().unwrap();

    let denial = denial.identity().unwrap().get();
    let disclosure = disclosure.outcome_identity().unwrap().get();
    assert_ne!(denial, disclosure);
    assert_ne!(denial, commit.get());
    assert_ne!(disclosure, commit.get());
}
