//! The action requirement can revoke independently while facet disclosure stays allowed.
use super::super::super::{host, schema::*};
use super::super::{grant_status::SetStatus, metadata, Resource};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, primary_graph::*,
};

pub(super) fn run(
    host: &host::Host,
    actor: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    issuer: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    scope: &WorthQueryRequestScope,
    handle: &mut WorthQueryApplicationResource<CapabilitySchema, Resource>,
) {
    let request = host.application.runtime().request(actor, scope);
    let resources = request.resources();
    resources.advance(handle).unwrap();
    set_status(host, issuer, scope, false, 0x9181_8012);
    let contacts = host.contacts.resource.counts();
    let clock = host.contacts.resource_clock.observations();
    let denied = resources
        .advance(handle)
        .expect_err("real action grant was revoked");
    assert_eq!(
        denied.action_authorization_denial_kind(),
        Some(WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
        ))
    );
    assert_eq!(host.contacts.resource.counts(), contacts);
    assert_eq!(host.contacts.resource_clock.observations(), clock);
    assert!(
        !resources
            .result_available_governed(
                &handle.receipt(),
                metadata::admit(host, actor, scope).unwrap()
            )
            .unwrap(),
        "the separate metadata grant is still live"
    );
    set_status(host, issuer, scope, true, 0x9181_8013);
    resources.advance(handle).unwrap();
    let before = host.contacts.resource.counts();
    host.contacts
        .resource
        .cleanup_pending
        .store(true, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        resources.revalidate(handle).unwrap_err().kind(),
        WorthQueryResourceStopKind::Cleanup,
        "the real replacement retains its old provider cleanup"
    );
    assert_eq!(
        host.contacts.resource.counts(),
        (before.0, before.1, before.2 + 1)
    );
    assert!(host.contacts.resource.send(0, 999).is_err());
    host.contacts
        .resource
        .cleanup_pending
        .store(false, std::sync::atomic::Ordering::SeqCst);
    resources.advance(handle).unwrap();
    assert_eq!(
        host.contacts.resource.counts(),
        (before.0 + 1, before.1, before.2 + 2),
        "the retained old provider closes before the successor is issued"
    );
    host.contacts.resource.send(before.0, 1212).unwrap();
    resources.advance(handle).unwrap();
    assert_eq!(
        resources
            .with_result_governed(
                &handle.receipt(),
                metadata::admit(host, actor, scope).unwrap(),
                |value| *value,
            )
            .unwrap(),
        Some(1212)
    );
}

fn set_status(
    host: &host::Host,
    issuer: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    scope: &WorthQueryRequestScope,
    active: bool,
    key: u64,
) {
    let outcome = host
        .application
        .runtime()
        .request(issuer, scope)
        .advanced()
        .mutate(SetStatus {
            identity: "resource-action".to_owned(),
            active,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            &host.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
