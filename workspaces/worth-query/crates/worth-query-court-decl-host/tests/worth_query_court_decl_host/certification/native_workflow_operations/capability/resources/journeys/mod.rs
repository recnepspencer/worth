pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_workflow_operations::capability::resources::journeys::*;

#[test]
fn installed_resource_finite_pool_keeps_retained_result_separate_from_reused_native_node() {
    let host = published();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let request = host.application.runtime().request(&actor, &scope);
    let resources = request.resources();
    let mut a = start(&resources, 10);
    let mut b = start(&resources, 20);
    let stop = resources
        .start::<Resource>(30)
        .err()
        .expect("two actual Native slots are full");
    let (pending, cause) = stop.into_parts();
    assert_eq!(cause.kind(), WorthQueryResourceStopKind::Capacity);
    let mut pending = pending.expect("allocated row remains owned after Native slot refusal");
    resources.close(&mut pending).unwrap();
    assert_eq!(host.contacts.resource.counts(), (2, 0, 0));
    assert_eq!(host.contacts.resource.input(0), 10);
    assert_eq!(host.contacts.resource.input(1), 20);
    resources.revalidate(&mut b).unwrap();
    assert!(host.contacts.resource.send(1, 999).is_err());
    let contacts = host.contacts.resource.counts();
    assert_eq!(contacts, (3, 0, 1));
    resources.revalidate(&mut b).unwrap();
    assert_eq!(
        host.contacts.resource.counts(),
        contacts,
        "actual coalescence preserves the existing winner provider"
    );
    host.contacts.resource.send(2, 202).unwrap();

    host.contacts.resource.send(0, 101).unwrap();
    resources.advance(&mut a).unwrap();
    let original = a.receipt();
    assert_eq!(
        resources
            .with_result_governed(
                &original,
                metadata::admit(&host, &actor, &scope).unwrap(),
                |value| *value
            )
            .unwrap(),
        Some(101)
    );
    let mut c = start(&resources, 30);
    host.contacts.resource.send(3, 303).unwrap();
    let full = resources
        .advance(&mut c)
        .expect_err("retained result capacity is independent of live nodes");
    assert_eq!(full.kind(), WorthQueryResourceStopKind::Capacity);
    assert_eq!(host.contacts.resource.counts().1, 2);

    resources.close(&mut a).unwrap();
    resources.advance(&mut c).unwrap();
    assert_eq!(
        host.contacts.resource.counts().1,
        2,
        "admitted scalar is retained across result-capacity refusal"
    );
    assert_eq!(
        resources
            .with_result_governed(
                &c.receipt(),
                metadata::admit(&host, &actor, &scope).unwrap(),
                |value| *value
            )
            .unwrap(),
        Some(303)
    );
    assert!(resources
        .result_available_governed(&original, metadata::admit(&host, &actor, &scope).unwrap())
        .is_err());
    let detailed = resources
        .close_detailed_governed(&mut b, metadata::admit(&host, &actor, &scope).unwrap())
        .unwrap();
    assert_eq!(
        detailed.detail(),
        Some(&WorthQueryResourceCleanupDetail {
            provider_closed: true,
            queue_drained: true,
            native_released: true,
        })
    );
    resources.close(&mut c).unwrap();
    assert!(
        host.contacts.resource.send(0, 999).is_err(),
        "late old writer cannot reach successor"
    );
    source_binding::run(&host, &actor, &scope);
}

#[test]
fn resource_verifier_rejects_forged_frame_without_creating_a_fulfilled_result() {
    let host = published();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let request = host.application.runtime().request(&actor, &scope);
    let resources = request.resources();
    let mut handle = start(&resources, 77);
    host.contacts.resource.send_forged(0).unwrap();
    assert!(resources.advance(&mut handle).is_err());
    assert_eq!(host.contacts.resource.counts(), (1, 1, 0));
    assert!(!resources
        .result_available_governed(
            &handle.receipt(),
            metadata::admit(&host, &actor, &scope).unwrap()
        )
        .unwrap());
    assert_eq!(
        resources
            .state_governed(
                &handle.receipt(),
                metadata::admit(&host, &actor, &scope).unwrap()
            )
            .unwrap(),
        WorthQueryResourceState::AwaitingResult
    );
    resources.close(&mut handle).unwrap();
    assert_eq!(host.contacts.resource.counts().2, 1);
    let mut failed = start(&resources, 78);
    host.contacts.resource.send_domain_failure(1).unwrap();
    resources.advance(&mut failed).unwrap();
    assert_eq!(
        resources
            .state_governed(
                &failed.receipt(),
                metadata::admit(&host, &actor, &scope).unwrap()
            )
            .unwrap(),
        WorthQueryResourceState::DomainFailureCommitted
    );
    assert_eq!(
        resources
            .with_domain_failure_governed(
                &failed.receipt(),
                metadata::admit(&host, &actor, &scope).unwrap(),
                |value| value.clone()
            )
            .unwrap(),
        Some("provider-domain-failure".to_owned())
    );
    assert_eq!(
        resources
            .with_result_governed(
                &failed.receipt(),
                metadata::admit(&host, &actor, &scope).unwrap(),
                |value| *value
            )
            .unwrap(),
        None,
        "typed domain failure does not become a successful result"
    );
    resources.close(&mut failed).unwrap();
}

#[test]
fn fresh_resource_metadata_revocation_precedes_provider_access_but_does_not_disable_close() {
    let host = published();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let issuer = host::authenticate(&host, 2, &scope);
    let request = host.application.runtime().request(&actor, &scope);
    let resources = request.resources();
    let mut handle = start(&resources, 12);
    let receipt = handle.receipt();
    assert!(
        resources.result_available(&receipt).is_err(),
        "action permission is not metadata permission"
    );
    let retained_metadata = metadata::admit(&host, &actor, &scope).unwrap();
    assert!(!resources
        .result_available_governed(&receipt, metadata::admit(&host, &actor, &scope).unwrap())
        .unwrap());
    let WorthQueryCapabilityCommandAdvance::Completed(revoked) =
        grant_commands::revoke(&host, &issuer, &scope, 0x9181_8010)
    else {
        panic!("actual grant revocation")
    };
    revoked
        .close(&host.application.runtime().request(&issuer, &scope))
        .unwrap_or_else(|(_, cause)| panic!("revocation close: {cause:?}"));
    let before = host.contacts.resource.counts();
    let denied_facet = resources
        .result_available_governed(&receipt, retained_metadata)
        .expect_err("the original metadata grant is revoked");
    assert_eq!(denied_facet.action_authorization_denial_kind(), None);
    assert_eq!(host.contacts.resource.counts(), before);
    let restored = host
        .application
        .runtime()
        .request(&issuer, &scope)
        .advanced()
        .mutate(super::grant_status::SetStatus {
            identity: "source-child".to_owned(),
            active: true,
        })
        .without_source()
        .idempotency(&0x9181_8011_u64)
        .execute_in_program(
            &host.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        restored,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(!resources
        .result_available_governed(&receipt, metadata::admit(&host, &actor, &scope).unwrap())
        .unwrap());
    action_authorization::run(&host, &actor, &issuer, &scope, &mut handle);
    resources.close(&mut handle).unwrap();
    assert_eq!(host.contacts.resource.counts().2, before.2 + 3);
}

#[test]
fn dropped_public_resource_is_reclaimed_only_by_exact_receipt_after_pending_cleanup() {
    let mut host = published();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let request = host.application.runtime().request(&actor, &scope);
    let resources = request.resources();
    let mut handle = start(&resources, 14);
    let receipt = handle.receipt();
    host.contacts
        .resource
        .cleanup_pending
        .store(true, Ordering::SeqCst);
    assert_eq!(
        resources
            .close(&mut handle)
            .expect_err("provider reports real pending cleanup")
            .kind(),
        WorthQueryResourceStopKind::Cleanup
    );
    drop(handle);
    assert_eq!(
        host.application
            .close_conditional_runtime()
            .expect_err("actual shutdown preserves pending resource owner")
            .kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::ResourceCleanupRetained
    );
    let request = host.application.runtime().request(&actor, &scope);
    let resources = request.resources();
    let mut reclaimed = resources
        .reclaim_governed(&receipt, metadata::admit(&host, &actor, &scope).unwrap())
        .unwrap_or_else(|stop| panic!("reclaim: {:?}", stop.into_parts().1));
    host.contacts
        .resource
        .cleanup_pending
        .store(false, Ordering::SeqCst);
    resources.close(&mut reclaimed).unwrap();
    let mut successor = start(&resources, 15);
    assert!(resources
        .reclaim_governed(&receipt, metadata::admit(&host, &actor, &scope).unwrap())
        .is_err());
    assert_eq!(host.contacts.resource.counts().0, 2);
    resources.close(&mut successor).unwrap();
    host.application.close_conditional_runtime().unwrap();
    assert_eq!(
        host.application
            .runtime()
            .request(&actor, &scope)
            .resources()
            .state(&receipt)
            .unwrap_err()
            .kind(),
        WorthQueryResourceStopKind::Unavailable
    );
    let (_, stopped) = host
        .application
        .runtime()
        .request(&actor, &scope)
        .resources()
        .start::<Resource>(999)
        .err()
        .expect("closed installation cannot restart its old pool")
        .into_parts();
    assert_eq!(stopped.kind(), WorthQueryResourceStopKind::Unavailable);
    assert_eq!(host.contacts.resource.counts().0, 2);
    drop(host);
    let foreign = published();
    let fresh = request_scope();
    let actor = host::authenticate(&foreign, 1, &fresh);
    let expired = foreign
        .application
        .runtime()
        .request(&actor, &fresh)
        .resources()
        .result_available_governed(&receipt, metadata::admit(&foreign, &actor, &fresh).unwrap())
        .expect_err("old original branch cannot be adopted by a restarted runtime");
    assert_eq!(
        expired.selection_denial(),
        Some(&WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner
        ))
    );
    assert_eq!(foreign.contacts.resource.counts(), (0, 0, 0));
    assert_eq!(foreign.contacts.resource_clock.observations(), 0);
    scope_replacement::run();
}

#[test]
fn original_resource_control_refusal_keeps_real_custody_and_later_ingress_uses_fresh_request() {
    control::run();
}
