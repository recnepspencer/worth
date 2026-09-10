use super::*;

#[test]
fn cancelled_request_cannot_reuse_otherwise_current_authority() {
    let world = installed_authorization_world(true);
    let live_request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &live_request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &live_request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &live_request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled_request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    cancellation.cancel();

    let denial = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &cancelled_request,
        )
        .err()
        .expect("cancelled request must deny");
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
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let expiring = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
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
        .invariant
        .project_admitted_operation(&expiring, |_, _| expired_projection_ran.set(true))
        .err()
        .expect("expired admission must deny before projection");
    assert_eq!(
        expired_projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication
        )
    );
    assert!(!expired_projection_ran.get());

    let cancellation = WorthQueryCancellationSource::new();
    let cancellable_request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &cancellable_request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &cancellable_request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let cancellable = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &cancellable_request,
        )
        .unwrap();
    cancellation.cancel();
    assert_eq!(
        cancellable.validate_current_authority().unwrap_err().kind(),
        WorthQueryOperationAuthorizationDenialKind::Cancelled
    );
    let cancelled_projection_ran = Cell::new(false);
    let cancelled_projection = world
        .invariant
        .project_admitted_operation(&cancellable, |_, _| cancelled_projection_ran.set(true))
        .err()
        .expect("cancelled admission must deny before projection");
    assert_eq!(
        cancelled_projection.kind(),
        WorthQueryOperationProjectionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Cancelled
        )
    );
    assert!(!cancelled_projection_ran.get());
}
