pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::wait_revocation::*;

#[test]
fn revoked_approval_grant_cannot_authorize_the_waiting_effect() {
    assert_grant_change_denies_wait(
        &["revoked"],
        2_100,
        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable,
    );
}

#[test]
fn reactivated_approval_grant_does_not_revive_the_published_approval() {
    assert_grant_change_denies_wait(
        &["revoked", "active"],
        2_200,
        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalAuthorityDenied,
    );
}
