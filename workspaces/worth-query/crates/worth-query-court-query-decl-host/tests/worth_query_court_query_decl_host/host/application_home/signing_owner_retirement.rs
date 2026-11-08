//! The intact signer registry follows its original Query owner through close.
use super::*;
use worth_query_host::facade::{
    admission::authentication_event::WorthQueryAuthenticationEventIntent,
    application_installation::WorthQuerySigningOwnerInstallationDenial,
    primary_graph::{
        WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{authenticate_operator, block_on, request_scope},
    workflow::authentication::*,
};
mod installation;

#[test]
fn installed_signing_owner_survives_close_refusal_and_retires_only_at_explicit_reclaim() {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = pending_support::open(execution, contacts.clone());
    let drops = Arc::new(AtomicUsize::new(0));
    let authentication =
        installation::install(home.runtime().runtime().installed_schema(), drops.clone());
    let original_owner = Arc::downgrade(&authentication);
    let mut retained_runtime = home.runtime().clone();
    let revision = *retained_runtime.installed_program().revision();
    retained_runtime
        .install_signing_owner(revision, authentication.signing_owner())
        .unwrap();
    drop(authentication);
    let refused = home
        .close(&execution)
        .err()
        .expect("the original runtime alias refuses");
    assert!(matches!(
        refused.query_denial(),
        Some(Denial::Busy(Owner::ProgramRuntime))
    ));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert!(original_owner.upgrade().is_some());
    let home = refused.resume().unwrap();
    let (_, scope) = pending_support::scope();
    assert_eq!(pending_support::balance(home.runtime(), &scope), 7);
    let (other_owner, _) = install_certification_authentication_with_clock(
        home.runtime().runtime().installed_schema(),
    );
    assert_eq!(
        retained_runtime.install_signing_owner(revision, other_owner.signing_owner()),
        Err(WorthQuerySigningOwnerInstallationDenial::AlreadyInstalled),
        "the same installed registry survives refusal and resume",
    );
    drop(other_owner);
    drop(retained_runtime);
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("installed signer close: {refused:?}"));
    assert!(closed.is_closed());
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        0,
        "retirement never calls business"
    );
    assert_eq!(
        drops.load(Ordering::SeqCst),
        0,
        "budgeted close must retain the final supplier"
    );
    assert!(original_owner.upgrade().is_some());
    closed.reclaim();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(original_owner.upgrade().is_none());
}

#[test]
fn home_close_does_not_revoke_a_shared_external_authentication_service() {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = pending_support::open(execution, Arc::new(AtomicUsize::new(0)));
    let (authentication, _) = install_certification_authentication_with_clock(
        home.runtime().runtime().installed_schema(),
    );
    let mut configuration = home.runtime().clone();
    let revision = *configuration.installed_program().revision();
    configuration
        .install_signing_owner(revision, authentication.signing_owner())
        .unwrap();
    drop(configuration);
    let scope = request_scope();
    let principal = authenticate_operator(home.runtime().runtime().installed_schema(), &scope);
    let intent = WorthQueryAuthenticationEventIntent::new(
        "certification-account-login",
        worth_foundational::facade::CanonicalDigestId::new([0x11; 32]),
        worth_foundational::facade::CanonicalDigestId::new([0x22; 32]),
    )
    .unwrap();
    let event =
        block_on(authentication.authenticate((), &principal, intent.clone(), &scope)).unwrap();
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("shared signer close: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
    let consumed = authentication
        .consume_for_signing(&event, &principal, &intent, &scope)
        .unwrap();
    assert_eq!(consumed.principal(), principal.identity());
    let fresh_event =
        block_on(authentication.authenticate((), &principal, intent.clone(), &scope)).unwrap();
    let fresh = authentication
        .consume_for_signing(&fresh_event, &principal, &intent, &scope)
        .unwrap();
    assert_eq!(fresh.principal(), principal.identity());
    assert_ne!(fresh.event_reference(), consumed.event_reference());
}
