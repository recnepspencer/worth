//! Early partial artifacts must retain their actual owning refusal.
use super::*;
use worth_query_host::facade::{
    primary_graph::WorthQueryPrimaryGraphInstallationDenialKind,
    WorthQueryApplicationHomeOpenRefused,
};

#[test]
fn initializer_refusal_without_canonical_indexes_reopens_at_require_recovered_without_replay() {
    let location = fixture::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let called = callbacks.clone();
    let request = fixture::request_with_initializer(
        location.clone(),
        build,
        contacts.clone(),
        move |_, _, _| {
            called.fetch_add(1, Ordering::SeqCst);
            panic!("court initializer stopped before any seed publication")
        },
    );
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("the actual initializer refuses");
    assert!(matches!(
        &refused,
        WorthQueryApplicationHomeOpenRefused::InitialState {
            cause: worth_query_execution::facade::application_home_integration::QueryHomeInitialStateDenial::Panicked,
            ..
        }
    ));
    refused
        .close(&execution)
        .unwrap_or_else(|refused| panic!("original owning close: {refused:?}"));
    let request =
        fixture::request_with_initializer(location.clone(), build, contacts.clone(), |_, _, _| {
            panic!("partial existing artifacts cannot replay initialization")
        });
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("actual recovered index definitions are absent");
    match &refused {
        WorthQueryApplicationHomeOpenRefused::ExistingNativeFinalization { query, .. } => {
            assert_eq!(
                query.reason().kind(),
                WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected
            );
            assert!(
                query.reason().subject().starts_with(
                    "recovered runtime omitted Query index definition 'application-principal."
                ),
                "{:?}",
                query.reason()
            );
        }
        other => {
            panic!("RequireRecovered must refuse before World/selected publication: {other:?}")
        }
    }
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    let competing = fixture::request_with_initializer(location, build, contacts, |_, _, _| {
        panic!("held lease cannot initialize")
    });
    let competing = with_bootstrap_advancement_from_request(execution, |phase| {
        competing.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("the original refusal still holds its mutation lease");
    assert!(matches!(
        &competing,
        WorthQueryApplicationHomeOpenRefused::Physical { .. }
    ));
    assert!(competing.detail().contains("MutationOwnerContended"));
    competing
        .close(&execution)
        .unwrap_or_else(|refused| panic!("competing request disposal: {refused:?}"));
    refused
        .close(&execution)
        .unwrap_or_else(|refused| panic!("existing owning close: {refused:?}"));
}
