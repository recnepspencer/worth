//! Revocation must also fence disclosure through the original active/completed owners.
use super::{grant_commands, host, program::*, schema::*};
use crate::document_retention_model::operator_identity::request_scope;
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[test]
fn retained_source_inspection_rechecks_capability_after_real_grant_revocation() {
    let host = host::install();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let issuer = host::authenticate(&host, 2, &scope);
    let request = host.application.runtime().request(&actor, &scope);
    let capture = |days, key| {
        request
            .mutate(Original {
                input: SetRetentionInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                    retention_days: days,
                },
            })
            .without_source()
            .idempotency_owned(key)
            .unwrap_or_else(|(_, _, denial)| panic!("capture: {denial:?}"))
            .retain_operation::<Program, Action>(&host.application)
            .unwrap_or_else(|(_, denial)| panic!("owner: {denial:?}"))
    };
    let pending = capture(18, 0x9176_5191_u64);
    pending.assess_current_authorization(&request).unwrap();
    assert_eq!(host.contacts.counts(), (0, 0));
    let WorthQuerySourceOnlyAdvance::Completed(completed) =
        capture(8, 0x9176_5192_u64).advance(&request)
    else {
        panic!("actual source completion")
    };
    completed.assess_current_authorization(&request).unwrap();
    let WorthQueryCapabilityCommandAdvance::Completed(revoked) =
        grant_commands::revoke(&host, &issuer, &scope, 0x9176_5193)
    else {
        panic!("real revocation")
    };
    revoked
        .close(&host.application.runtime().request(&issuer, &scope))
        .unwrap_or_else(|(_, denial)| panic!("revoke close: {denial:?}"));
    assert_eq!(super::readback::grant_status(&host), 0);
    let fresh_scope = request_scope();
    let fresh_actor = host::authenticate(&host, 1, &fresh_scope);
    let fresh = host
        .application
        .runtime()
        .request(&fresh_actor, &fresh_scope);
    let before = fresh
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    for result in [
        pending.assess_current_authorization(&fresh),
        completed.assess_current_authorization(&fresh),
    ] {
        assert!(
            matches!(result, Err(WorthQueryApplicationRequestMutationDenial::Authorization(cause)) if cause.kind() == WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing))
        );
    }
    assert_eq!(pending.posture(), WorthQuerySourceOnlyPosture::Initial);
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &before
    );
    pending
        .close(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("pending custody: {denial:?}"));
    completed
        .close(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("completed custody: {denial:?}"));
}
