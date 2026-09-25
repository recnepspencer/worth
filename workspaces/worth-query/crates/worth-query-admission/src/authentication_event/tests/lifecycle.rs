use std::num::NonZeroU32;

use super::*;

#[test]
fn named_clock_expiry_reuse_revocation_and_capacity_are_owner_decisions() {
    let schema = installed_schema();
    let principal = principal(&schema);
    let source = ClockSource::new();
    let policy = WorthQueryAuthenticationEventPolicy::new(
        Duration::from_nanos(10),
        WorthQueryAuthenticationEventReuse::AtMost(NonZeroU32::new(2).unwrap()),
    )
    .unwrap();
    let owner = install_authentication_event_owner::<TestSchema, AuthenticationClock, _, _>(
        &schema,
        source.clone(),
        EventVerifier {
            observed_nonces: Arc::new(Mutex::new(Vec::new())),
        },
        policy,
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap();
    let intended = intent("approve-payment", 7);
    let first = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .ok()
    .unwrap();
    assert_eq!(
        block_on(owner.authenticate(
            EventCredential::Valid,
            &principal,
            intended.clone(),
            &scope()
        ))
        .err(),
        Some(WorthQueryAuthenticationEventDenial::CapacityExceeded)
    );
    assert!(owner
        .consume_for_signing(&first, &principal, &intended, &scope())
        .is_ok());
    assert!(owner
        .consume_for_signing(&first, &principal, &intended, &scope())
        .is_ok());
    assert_eq!(
        owner
            .consume_for_signing(&first, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::EventUnavailable)
    );
    let revoked = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .ok()
    .unwrap();
    owner.revoke(&revoked).unwrap();
    assert_eq!(
        owner
            .consume_for_signing(&revoked, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::EventRevoked)
    );
    let expiring = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .ok()
    .unwrap();
    source.advance(10);
    assert_eq!(
        owner
            .consume_for_signing(&expiring, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::EventExpired)
    );
    let administratively_revoked = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .ok()
    .unwrap();
    assert_eq!(
        owner.revoke_outstanding_for_principal(principal.identity()),
        1
    );
    assert_eq!(
        owner
            .consume_for_signing(&administratively_revoked, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::EventRevoked)
    );
    let checked_then_delayed = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .ok()
    .unwrap();
    let consumed = owner
        .consume_for_signing(&checked_then_delayed, &principal, &intended, &scope())
        .ok()
        .unwrap();
    source.advance(10);
    assert_eq!(
        owner
            .readmit_consumed_for_signing(consumed, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::EventExpired)
    );
}

#[test]
fn rejected_credential_and_clock_regression_fail_closed() {
    let schema = installed_schema();
    let principal = principal(&schema);
    let source = ClockSource::new();
    let policy = WorthQueryAuthenticationEventPolicy::new(
        Duration::from_nanos(100),
        WorthQueryAuthenticationEventReuse::SingleUse,
    )
    .unwrap();
    let owner = install_authentication_event_owner::<TestSchema, AuthenticationClock, _, _>(
        &schema,
        source.clone(),
        EventVerifier {
            observed_nonces: Arc::new(Mutex::new(Vec::new())),
        },
        policy,
        NonZeroUsize::new(2).unwrap(),
    )
    .unwrap();
    let intended = intent("approve-payment", 7);
    assert_eq!(
        block_on(owner.authenticate(
            EventCredential::Rejected,
            &principal,
            intended.clone(),
            &scope()
        ))
        .err(),
        Some(WorthQueryAuthenticationEventDenial::VerifierFailed(
            WorthQueryAuthenticationEventVerifierFailure::CredentialRejected
        ))
    );
    let event = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .ok()
    .unwrap();
    source.now.store(999, Ordering::SeqCst);
    assert_eq!(
        owner
            .consume_for_signing(&event, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::ClockRegressed)
    );
    source.now.store(1_000, Ordering::SeqCst);
    source.changed_identity.store(true, Ordering::SeqCst);
    assert_eq!(
        owner
            .consume_for_signing(&event, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::ClockIdentityChanged)
    );
}

#[test]
fn unfinished_verification_reserves_capacity_and_drop_releases_it() {
    let schema = installed_schema();
    let principal = principal(&schema);
    let policy = WorthQueryAuthenticationEventPolicy::new(
        Duration::from_nanos(100),
        WorthQueryAuthenticationEventReuse::SingleUse,
    )
    .unwrap();
    let owner = install_authentication_event_owner::<TestSchema, AuthenticationClock, _, _>(
        &schema,
        ClockSource::new(),
        EventVerifier {
            observed_nonces: Arc::new(Mutex::new(Vec::new())),
        },
        policy,
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap();
    let intended = intent("approve-payment", 7);
    let first_scope = scope();
    let mut unfinished = Box::pin(owner.authenticate(
        EventCredential::Pending,
        &principal,
        intended.clone(),
        &first_scope,
    ));
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    assert!(matches!(
        unfinished.as_mut().poll(&mut context),
        Poll::Pending
    ));
    assert_eq!(
        block_on(owner.authenticate(
            EventCredential::Valid,
            &principal,
            intended.clone(),
            &scope()
        ))
        .err(),
        Some(WorthQueryAuthenticationEventDenial::CapacityExceeded)
    );
    drop(unfinished);
    assert!(
        block_on(owner.authenticate(EventCredential::Valid, &principal, intended, &scope()))
            .is_ok()
    );
}

#[test]
fn revocation_during_external_verification_cannot_later_issue_event() {
    let schema = installed_schema();
    let principal = principal(&schema);
    let policy = WorthQueryAuthenticationEventPolicy::new(
        Duration::from_nanos(100),
        WorthQueryAuthenticationEventReuse::SingleUse,
    )
    .unwrap();
    let owner = install_authentication_event_owner::<TestSchema, AuthenticationClock, _, _>(
        &schema,
        ClockSource::new(),
        EventVerifier {
            observed_nonces: Arc::new(Mutex::new(Vec::new())),
        },
        policy,
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap();
    let intended = intent("approve-payment", 7);
    let release = Arc::new(AtomicBool::new(false));
    let request = scope();
    let mut unfinished = Box::pin(owner.authenticate(
        EventCredential::WaitFor(Arc::clone(&release)),
        &principal,
        intended,
        &request,
    ));
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    assert!(matches!(
        unfinished.as_mut().poll(&mut context),
        Poll::Pending
    ));
    assert_eq!(
        owner.revoke_outstanding_for_principal(principal.identity()),
        1
    );
    release.store(true, Ordering::SeqCst);
    assert!(matches!(
        unfinished.as_mut().poll(&mut context),
        Poll::Ready(Err(WorthQueryAuthenticationEventDenial::EventRevoked))
    ));
}

#[test]
fn revocation_after_consumption_denies_publication_readmission() {
    let schema = installed_schema();
    let principal = principal(&schema);
    let policy = WorthQueryAuthenticationEventPolicy::new(
        Duration::from_nanos(100),
        WorthQueryAuthenticationEventReuse::SingleUse,
    )
    .unwrap();
    let owner = Arc::new(
        install_authentication_event_owner::<TestSchema, AuthenticationClock, _, _>(
            &schema,
            ClockSource::new(),
            EventVerifier {
                observed_nonces: Arc::new(Mutex::new(Vec::new())),
            },
            policy,
            NonZeroUsize::new(1).unwrap(),
        )
        .unwrap(),
    );
    let intended = intent("approve-payment", 7);
    let event = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .unwrap();
    let consumed = owner
        .signing_owner()
        .consume_for_signing(&event, &principal, &intended, &scope())
        .unwrap();
    owner.revoke(&event).unwrap();
    assert_eq!(
        owner
            .signing_owner()
            .readmit_for_publication(consumed)
            .err(),
        Some(WorthQueryAuthenticationEventDenial::EventRevoked)
    );
}
