//! Actual authored Host startup is a POSTBOOTSTRAP boundary, not the Store C0 proof.
use super::pending_support::{balance, scope, set};
use super::*;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationRelationIntegrity, ApplicationSchemaMember},
};
use worth_query_host::facade::{
    primary_graph::{
        WorthQueryHomeRetirementDenial as CloseDenial,
        WorthQueryOperationHomeRetirementDenial as HandlerDenial,
    },
    WorthQueryApplicationHomeCheckpointLimits,
};
#[test]
fn authored_fresh_home_postbootstrap_complete_and_automatic_maintenance_preserve_business_value() {
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let request = fixture::request(
        fixture::location(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        initialized.clone(),
        contacts.clone(),
    )
    .with_checkpoint_limits(WorthQueryApplicationHomeCheckpointLimits::new(
        std::num::NonZeroU64::MIN,
        std::num::NonZeroU64::new(64 << 20).unwrap(),
    ));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("authored fresh Home: {refused:?}"));
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    let installed = home
        .runtime()
        .runtime()
        .installed_schema()
        .installed_declaration();
    assert!(installed.members().iter().any(|member| matches!(member,
        ApplicationSchemaMember::Relation {relation,from,to,integrity}
            if relation == "UnusedAccountPeer" && from == "Account" && to == "Account"
                && *integrity == ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling().into()
    )), "the actual authored installed schema retains its unused physical relation");
    // The production opener already committed activation/principal seed rows.
    let first = home
        .checkpoint(&execution)
        .expect("first postbootstrap Complete");
    assert!(first.covered().0 > 0);
    let before = home.inspect_checkpoint(&execution).unwrap();
    assert!(!before.due && before.pending.is_none());
    assert!(before
        .journals
        .iter()
        .all(|journal| journal.head == journal.covered));
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), 7);
    let result = set(home.runtime(), &live, 0x4652455348, 8).unwrap();
    let worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { result, .. } = result else {
        panic!("genuine ordinary business publication required: {result:?}");
    };
    assert_eq!(
        result, 8,
        "maintenance cannot replace the actual business value"
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    let automatic = home.inspect_checkpoint(&execution).unwrap();
    assert!(!automatic.due && automatic.pending.is_none());
    assert!(automatic.journals[0].head > before.journals[0].head);
    for (old, current) in before.journals.iter().zip(&automatic.journals) {
        assert_eq!(old.name, current.name);
        assert!(current.head >= old.head);
        assert_eq!(
            current.covered, current.head,
            "both Native and World ACK frontiers: {}",
            current.name
        );
    }
    let manual = home.checkpoint(&execution).unwrap();
    assert_eq!(
        manual.generation(),
        first.generation() + 2,
        "one automatic and one manual selection share the actual anchor"
    );
    assert_eq!(manual.covered().0, automatic.journals[0].head);
    let settled = home.inspect_checkpoint(&execution).unwrap();
    assert!(!settled.due && settled.pending.is_none());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    // This fixture's real opaque supplier refuses before World seal. Preserve
    // that exact owning posture; do not accept arbitrary close failure or Closed.
    let refused = home
        .close(&execution)
        .err()
        .expect("opaque supplier retirement remains unsupported");
    assert!(
        matches!(refused.query_denial(), Some(CloseDenial::Handler {binding,cause:HandlerDenial::Unsupported})
        if *binding == fixture::native_mutation::SetBalanceBinding::IDENTITY)
    );
    let home = refused.resume().unwrap();
    assert!(home.is_directory());
    // Explicit ordinary abandonment makes no provider retirement/D11 claim.
    drop(home);
}
