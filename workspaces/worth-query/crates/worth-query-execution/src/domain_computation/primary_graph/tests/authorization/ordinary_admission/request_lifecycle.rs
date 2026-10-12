use super::*;

#[test]
fn cancelled_request_cannot_reuse_otherwise_current_authority() {
    let world = installed_authorization_world(true);
    let live_request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &live_request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &live_request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &live_request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled_request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    cancellation.cancel();

    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &cancelled_request,
                )
                .err()
                .expect("cancelled request must deny")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Cancelled
    );
}

#[test]
fn admitted_operation_retains_expiry_and_cancellation_authority() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(1), &request);
    let authentication_expires_at = external.expires_at();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let expiring = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(expiring.validate_current_authority().is_ok());
    let until_expiry = authentication_expires_at
        .duration_since(SystemTime::now())
        .unwrap_or_default();
    std::thread::sleep(until_expiry + Duration::from_millis(10));
    assert_eq!(
        expiring.validate_current_authority().unwrap_err().kind(),
        WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication
    );
    let expired_projection_ran = Cell::new(false);
    let expired_projection = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &expiring,
                    |_, _| expired_projection_ran.set(true),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("expired admission must deny before projection")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        expired_projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication
        )
    );
    assert!(!expired_projection_ran.get());

    let cancellation = WorthQueryCancellationSource::new();
    let cancellable_request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &cancellable_request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &cancellable_request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let cancellable = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &cancellable_request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    cancellation.cancel();
    assert_eq!(
        cancellable.validate_current_authority().unwrap_err().kind(),
        WorthQueryOperationAuthorizationDenialKind::Cancelled
    );
    let cancelled_projection_ran = Cell::new(false);
    let cancelled_projection = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &cancellable,
                    |_, _| cancelled_projection_ran.set(true),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("cancelled admission must deny before projection")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        cancelled_projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Cancelled
        )
    );
    assert!(!cancelled_projection_ran.get());
}
